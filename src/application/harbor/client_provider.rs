use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::application::oidc_service::OidcService;
use crate::domain::oidc::OidcClient;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::sync::Arc;
use uuid::Uuid;

const REDACTED_SECRET: &str = "${REDACTED}";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborClientPayload {
    pub client_id: String,
    pub client_secret: Option<String>,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub web_origins: Vec<String>,
}

pub struct ClientHarborProvider {
    oidc_service: Arc<OidcService>,
}

impl ClientHarborProvider {
    pub fn new(oidc_service: Arc<OidcService>) -> Self {
        Self { oidc_service }
    }
}

fn parse_client_scopes(scopes: &str) -> Result<Vec<String>> {
    match serde_json::from_str(scopes) {
        Ok(scopes) => Ok(scopes),
        Err(_) => {
            let legacy_scopes = scopes
                .split_whitespace()
                .map(str::trim)
                .filter(|scope| !scope.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            if legacy_scopes.is_empty() && !scopes.trim().is_empty() {
                return Err(Error::Validation("Invalid scopes payload".to_string()));
            }

            Ok(legacy_scopes)
        }
    }
}

#[async_trait]
impl HarborProvider for ClientHarborProvider {
    fn key(&self) -> &'static str {
        "client"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        if !resource.assets.is_empty() {
            return Err(Error::Validation(
                "Client bundles must not include assets".to_string(),
            ));
        }

        let payload: HarborClientPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid client bundle payload: {}", err)))?;

        if payload.client_id.trim().is_empty() {
            return Err(Error::Validation("Client id is required".to_string()));
        }

        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        let client_id = match scope {
            HarborScope::Client { client_id } => client_id.as_str(),
            _ => {
                return Err(Error::Validation(
                    "Client export requires client scope".to_string(),
                ))
            }
        };

        let client = self
            .oidc_service
            .find_client_by_client_id_with_secret(&realm_id, client_id)
            .await?
            .ok_or_else(|| Error::OidcClientNotFound(client_id.to_string()))?;

        let redirect_uris: Vec<String> = serde_json::from_str(&client.redirect_uris)
            .map_err(|_| Error::Validation("Invalid redirect_uris payload".to_string()))?;
        let scopes = parse_client_scopes(&client.scopes)?;
        let web_origins: Vec<String> = serde_json::from_str(&client.web_origins)
            .map_err(|_| Error::Validation("Invalid web_origins payload".to_string()))?;

        let client_secret = match policy {
            ExportPolicy::IncludeSecrets => client.client_secret.clone(),
            ExportPolicy::Redact => client
                .client_secret
                .as_ref()
                .map(|_| REDACTED_SECRET.to_string()),
        };

        let payload = HarborClientPayload {
            client_id: client.client_id,
            client_secret,
            redirect_uris,
            scopes,
            web_origins,
        };

        let data = to_value(&payload)
            .map_err(|err| Error::System(format!("Failed to serialize client: {}", err)))?;

        Ok(HarborResourceBundle {
            key: self.key().to_string(),
            data,
            assets: Vec::new(),
            meta: None,
        })
    }

    async fn import(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        resource: &HarborResourceBundle,
        conflict_policy: ConflictPolicy,
        dry_run: bool,
        mut tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResourceResult> {
        let scoped_client_id = match scope {
            HarborScope::Client { client_id } => client_id.as_str(),
            _ => {
                return Err(Error::Validation(
                    "Client import requires client scope".to_string(),
                ))
            }
        };

        let payload: HarborClientPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid client bundle payload: {}", err)))?;

        if payload.client_id != scoped_client_id {
            return Err(Error::Validation(
                "Client bundle id does not match import scope".to_string(),
            ));
        }

        let existing = self
            .oidc_service
            .find_client_by_client_id(&realm_id, scoped_client_id)
            .await?;

        if existing.is_some() {
            match conflict_policy {
                ConflictPolicy::Skip => {
                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "skipped".to_string(),
                        created: 0,
                        updated: 0,
                        errors: Vec::new(),
                        original_id: Some(scoped_client_id.to_string()),
                        renamed_to: None,
                    });
                }
                ConflictPolicy::Rename => {
                    let new_client_id =
                        resolve_available_client_id(&self.oidc_service, realm_id, scoped_client_id)
                            .await?;
                    let mut payload = payload;
                    payload.client_id = new_client_id.clone();
                    let mut result = import_new_client(
                        &self.oidc_service,
                        realm_id,
                        payload,
                        new_client_id.clone(),
                        dry_run,
                        tx.as_deref_mut(),
                    )
                    .await?;
                    result.original_id = Some(scoped_client_id.to_string());
                    result.renamed_to = Some(new_client_id);
                    return Ok(result);
                }
                ConflictPolicy::Overwrite => {}
            }
        }

        if dry_run {
            return Ok(HarborImportResourceResult {
                key: self.key().to_string(),
                status: "validated".to_string(),
                created: 0,
                updated: existing.map(|_| 1).unwrap_or(0),
                errors: Vec::new(),
                original_id: Some(scoped_client_id.to_string()),
                renamed_to: None,
            });
        }

        if let Some(mut client) = existing {
            apply_client_payload(&mut client, &payload, true)?;
            self.oidc_service
                .update_client_record_with_tx(&client, tx.as_deref_mut())
                .await?;
            return Ok(HarborImportResourceResult {
                key: self.key().to_string(),
                status: "updated".to_string(),
                created: 0,
                updated: 1,
                errors: Vec::new(),
                original_id: Some(scoped_client_id.to_string()),
                renamed_to: None,
            });
        }

        import_new_client(
            &self.oidc_service,
            realm_id,
            payload,
            scoped_client_id.to_string(),
            dry_run,
            tx,
        )
        .await
    }
}

fn apply_client_payload(
    client: &mut OidcClient,
    payload: &HarborClientPayload,
    preserve_secret_if_redacted: bool,
) -> Result<()> {
    client.redirect_uris =
        serde_json::to_string(&payload.redirect_uris).map_err(|e| Error::Unexpected(e.into()))?;
    client.scopes =
        serde_json::to_string(&payload.scopes).map_err(|e| Error::Unexpected(e.into()))?;
    client.web_origins =
        serde_json::to_string(&payload.web_origins).map_err(|e| Error::Unexpected(e.into()))?;

    let secret = payload.client_secret.as_deref().map(str::trim);
    match secret {
        Some(REDACTED_SECRET) | None if !preserve_secret_if_redacted => {
            client.client_secret = None;
        }
        Some(value) if !value.is_empty() => {
            client.client_secret = Some(value.to_string());
        }
        _ => {}
    }

    client.managed_by_config = false;
    Ok(())
}

async fn resolve_available_client_id(
    oidc_service: &OidcService,
    realm_id: Uuid,
    base: &str,
) -> Result<String> {
    for idx in 1..=1000 {
        let candidate = format!("{}-{}", base, idx);
        if oidc_service
            .find_client_by_client_id(&realm_id, &candidate)
            .await?
            .is_none()
        {
            return Ok(candidate);
        }
    }
    Err(Error::Validation(
        "Unable to generate unique client id".to_string(),
    ))
}

async fn import_new_client(
    oidc_service: &OidcService,
    realm_id: Uuid,
    payload: HarborClientPayload,
    client_id: String,
    dry_run: bool,
    tx: Option<&mut dyn Transaction>,
) -> Result<HarborImportResourceResult> {
    if dry_run {
        return Ok(HarborImportResourceResult {
            key: "client".to_string(),
            status: "validated".to_string(),
            created: 1,
            updated: 0,
            errors: Vec::new(),
            original_id: Some(payload.client_id.clone()),
            renamed_to: None,
        });
    }

    let mut client = OidcClient {
        id: Uuid::new_v4(),
        realm_id,
        client_id,
        client_secret: None,
        redirect_uris: "[]".to_string(),
        scopes: "[]".to_string(),
        web_origins: "[]".to_string(),
        managed_by_config: false,
    };

    apply_client_payload(&mut client, &payload, false)?;

    let _ = oidc_service
        .register_client_with_tx(&mut client, tx)
        .await?;

    Ok(HarborImportResourceResult {
        key: "client".to_string(),
        status: "created".to_string(),
        created: 1,
        updated: 0,
        errors: Vec::new(),
        original_id: Some(payload.client_id),
        renamed_to: None,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_client_scopes;

    #[test]
    fn parse_client_scopes_accepts_json_array() {
        let scopes = parse_client_scopes("[\"openid\",\"profile\"]").expect("json scopes");
        assert_eq!(scopes, vec!["openid", "profile"]);
    }

    #[test]
    fn parse_client_scopes_accepts_legacy_space_delimited_values() {
        let scopes =
            parse_client_scopes("openid profile email").expect("legacy space-delimited scopes");
        assert_eq!(scopes, vec!["openid", "profile", "email"]);
    }
}
