use crate::application::flow_manager::FlowManager;
use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::application::realm_service::{RealmService, UpdateRealmPayload};
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::sync::Arc;
use uuid::Uuid;

fn parse_role_ids(role_ids: &[String]) -> Result<Vec<Uuid>> {
    let mut parsed = Vec::with_capacity(role_ids.len());
    for role_id in role_ids {
        let id = Uuid::parse_str(role_id).map_err(|_| {
            Error::Validation(format!("Invalid default registration role id: {}", role_id))
        })?;
        parsed.push(id);
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HarborRealmFlowBindings {
    pub browser_flow_id: Option<String>,
    pub registration_flow_id: Option<String>,
    pub direct_grant_flow_id: Option<String>,
    pub reset_credentials_flow_id: Option<String>,
    pub invitation_flow_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborRealmPayload {
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
    pub pkce_required_public_clients: bool,
    pub lockout_threshold: i64,
    pub lockout_duration_secs: i64,
    #[serde(default)]
    pub invitation_resend_limit: Option<i64>,
    #[serde(default)]
    pub registration_enabled: Option<bool>,
    #[serde(default)]
    pub default_registration_role_ids: Option<Vec<String>>,
    #[serde(default)]
    pub flow_bindings: HarborRealmFlowBindings,
}

pub struct RealmHarborProvider {
    realm_service: Arc<RealmService>,
    flow_manager: Arc<FlowManager>,
}

impl RealmHarborProvider {
    pub fn new(realm_service: Arc<RealmService>, flow_manager: Arc<FlowManager>) -> Self {
        Self {
            realm_service,
            flow_manager,
        }
    }
}

#[async_trait]
impl HarborProvider for RealmHarborProvider {
    fn key(&self) -> &'static str {
        "realm"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        if !resource.assets.is_empty() {
            return Err(Error::Validation(
                "Realm bundles must not include assets".to_string(),
            ));
        }

        let payload: HarborRealmPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid realm bundle payload: {}", err)))?;

        for flow_id in [
            payload.flow_bindings.browser_flow_id.as_deref(),
            payload.flow_bindings.registration_flow_id.as_deref(),
            payload.flow_bindings.direct_grant_flow_id.as_deref(),
            payload.flow_bindings.reset_credentials_flow_id.as_deref(),
            payload.flow_bindings.invitation_flow_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            Uuid::parse_str(flow_id)
                .map_err(|_| Error::Validation(format!("Invalid flow binding id: {}", flow_id)))?;
        }
        if let Some(role_ids) = payload.default_registration_role_ids.as_ref() {
            parse_role_ids(role_ids)?;
        }

        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        _policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        if !matches!(scope, HarborScope::FullRealm) {
            return Err(Error::Validation(
                "Realm export requires full realm scope".to_string(),
            ));
        }

        let realm = self
            .realm_service
            .find_by_id(realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(realm_id.to_string()))?;

        let payload = HarborRealmPayload {
            access_token_ttl_secs: realm.access_token_ttl_secs,
            refresh_token_ttl_secs: realm.refresh_token_ttl_secs,
            pkce_required_public_clients: realm.pkce_required_public_clients,
            lockout_threshold: realm.lockout_threshold,
            lockout_duration_secs: realm.lockout_duration_secs,
            invitation_resend_limit: Some(realm.invitation_resend_limit),
            registration_enabled: Some(realm.registration_enabled),
            default_registration_role_ids: Some(
                realm
                    .default_registration_role_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
            ),
            flow_bindings: HarborRealmFlowBindings {
                browser_flow_id: realm.browser_flow_id,
                registration_flow_id: realm.registration_flow_id,
                direct_grant_flow_id: realm.direct_grant_flow_id,
                reset_credentials_flow_id: realm.reset_credentials_flow_id,
                invitation_flow_id: realm.invitation_flow_id,
            },
        };

        Ok(HarborResourceBundle {
            key: self.key().to_string(),
            data: to_value(payload)
                .map_err(|err| Error::System(format!("Failed to serialize realm: {}", err)))?,
            assets: Vec::new(),
            meta: None,
        })
    }

    async fn import(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        resource: &HarborResourceBundle,
        _conflict_policy: ConflictPolicy,
        dry_run: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResourceResult> {
        if !matches!(scope, HarborScope::FullRealm) {
            return Err(Error::Validation(
                "Realm import requires full realm scope".to_string(),
            ));
        }

        let payload: HarborRealmPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid realm bundle payload: {}", err)))?;

        if dry_run {
            return Ok(HarborImportResourceResult {
                key: self.key().to_string(),
                status: "validated".to_string(),
                created: 0,
                updated: 1,
                errors: Vec::new(),
                original_id: None,
                renamed_to: None,
            });
        }

        let HarborRealmPayload {
            access_token_ttl_secs,
            refresh_token_ttl_secs,
            pkce_required_public_clients,
            lockout_threshold,
            lockout_duration_secs,
            invitation_resend_limit,
            registration_enabled,
            default_registration_role_ids,
            flow_bindings,
        } = payload;

        let update_payload = UpdateRealmPayload {
            name: None,
            access_token_ttl_secs: Some(access_token_ttl_secs),
            refresh_token_ttl_secs: Some(refresh_token_ttl_secs),
            pkce_required_public_clients: Some(pkce_required_public_clients),
            lockout_threshold: Some(lockout_threshold),
            lockout_duration_secs: Some(lockout_duration_secs),
            invitation_resend_limit,
            registration_enabled,
            default_registration_role_ids: match default_registration_role_ids {
                Some(role_ids) => Some(parse_role_ids(&role_ids)?),
                None => None,
            },
            browser_flow_id: Some(parse_optional_uuid(flow_bindings.browser_flow_id.clone())?),
            registration_flow_id: Some(parse_optional_uuid(
                flow_bindings.registration_flow_id.clone(),
            )?),
            direct_grant_flow_id: Some(parse_optional_uuid(
                flow_bindings.direct_grant_flow_id.clone(),
            )?),
            reset_credentials_flow_id: Some(parse_optional_uuid(
                flow_bindings.reset_credentials_flow_id.clone(),
            )?),
            invitation_flow_id: Some(parse_optional_uuid(
                flow_bindings.invitation_flow_id.clone(),
            )?),
        };

        if let Some(tx) = tx {
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.browser_flow_id.as_deref(),
                Some(&mut *tx),
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.registration_flow_id.as_deref(),
                Some(&mut *tx),
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.direct_grant_flow_id.as_deref(),
                Some(&mut *tx),
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.reset_credentials_flow_id.as_deref(),
                Some(&mut *tx),
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.invitation_flow_id.as_deref(),
                Some(&mut *tx),
            )
            .await?;

            self.realm_service
                .update_realm_with_tx(realm_id, update_payload, Some(&mut *tx))
                .await?;
        } else {
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.browser_flow_id.as_deref(),
                None,
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.registration_flow_id.as_deref(),
                None,
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.direct_grant_flow_id.as_deref(),
                None,
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.reset_credentials_flow_id.as_deref(),
                None,
            )
            .await?;
            publish_bound_flow(
                &self.flow_manager,
                realm_id,
                flow_bindings.invitation_flow_id.as_deref(),
                None,
            )
            .await?;

            self.realm_service
                .update_realm_with_tx(realm_id, update_payload, None)
                .await?;
        }

        Ok(HarborImportResourceResult {
            key: self.key().to_string(),
            status: "updated".to_string(),
            created: 0,
            updated: 1,
            errors: Vec::new(),
            original_id: None,
            renamed_to: None,
        })
    }
}

async fn publish_bound_flow(
    flow_manager: &FlowManager,
    realm_id: Uuid,
    flow_id: Option<&str>,
    tx: Option<&mut dyn Transaction>,
) -> Result<()> {
    let Some(flow_id) = flow_id else {
        return Ok(());
    };

    let flow_id = Uuid::parse_str(flow_id)
        .map_err(|_| Error::Validation(format!("Invalid flow binding id: {}", flow_id)))?;

    flow_manager
        .publish_flow_with_tx(realm_id, flow_id, tx)
        .await?;
    Ok(())
}

fn parse_optional_uuid(value: Option<String>) -> Result<Option<Uuid>> {
    match value {
        Some(value) => {
            Ok(Some(Uuid::parse_str(&value).map_err(|_| {
                Error::Validation(format!("Invalid UUID: {}", value))
            })?))
        }
        None => Ok(None),
    }
}
