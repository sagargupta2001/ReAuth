use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::application::oidc_service::OidcService;
use crate::domain::pagination::PageRequest;
use crate::domain::role::Role;
use crate::error::{Error, Result};
use crate::ports::rbac_repository::RbacRepository;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborRolePayload {
    #[serde(default)]
    pub role_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

pub struct RoleHarborProvider {
    rbac_repo: Arc<dyn RbacRepository>,
    oidc_service: Arc<OidcService>,
}

impl RoleHarborProvider {
    pub fn new(rbac_repo: Arc<dyn RbacRepository>, oidc_service: Arc<OidcService>) -> Self {
        Self {
            rbac_repo,
            oidc_service,
        }
    }
}

#[async_trait]
impl HarborProvider for RoleHarborProvider {
    fn key(&self) -> &'static str {
        "role"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        if !resource.assets.is_empty() {
            return Err(Error::Validation(
                "Role bundles must not include assets".to_string(),
            ));
        }

        let payload: HarborRolePayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid role bundle payload: {}", err)))?;

        if payload.name.trim().is_empty() {
            return Err(Error::Validation("Role name is required".to_string()));
        }

        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        _policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        let role_id = match scope {
            HarborScope::Role { role_id } => *role_id,
            _ => {
                return Err(Error::Validation(
                    "Role export requires role scope".to_string(),
                ))
            }
        };

        let role = self
            .rbac_repo
            .find_role_by_id(&role_id)
            .await?
            .ok_or_else(|| Error::NotFound("Role not found".to_string()))?;

        if role.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Role belongs to different realm".to_string(),
            ));
        }

        let client_id = match role.client_id {
            Some(client_uuid) => Some(self.oidc_service.get_client(client_uuid).await?.client_id),
            None => None,
        };

        let mut permissions = self.rbac_repo.get_permissions_for_role(&role.id).await?;
        permissions.sort();

        let payload = HarborRolePayload {
            role_id: Some(role.id.to_string()),
            name: role.name,
            description: role.description,
            client_id,
            permissions,
        };

        let data = to_value(&payload)
            .map_err(|err| Error::System(format!("Failed to serialize role: {}", err)))?;

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
        tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResourceResult> {
        let fallback_role_id = match scope {
            HarborScope::Role { role_id } => *role_id,
            _ => {
                return Err(Error::Validation(
                    "Role import requires role scope".to_string(),
                ))
            }
        };

        let mut payload: HarborRolePayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid role bundle payload: {}", err)))?;

        let namespace_client_id =
            resolve_client_uuid(&self.oidc_service, realm_id, payload.client_id.as_deref()).await?;
        let existing = find_role_in_namespace(
            &*self.rbac_repo,
            realm_id,
            namespace_client_id,
            &payload.name,
        )
        .await?;

        if let Some(existing) = existing.clone() {
            match conflict_policy {
                ConflictPolicy::Skip => {
                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "skipped".to_string(),
                        created: 0,
                        updated: 0,
                        errors: Vec::new(),
                        original_id: Some(payload.name),
                        renamed_to: None,
                    });
                }
                ConflictPolicy::Rename => {
                    let renamed = resolve_available_role_name(
                        &*self.rbac_repo,
                        realm_id,
                        namespace_client_id,
                        &payload.name,
                    )
                    .await?;
                    let original_name = payload.name.clone();
                    payload.name = renamed.clone();
                    let result = import_new_role(
                        &*self.rbac_repo,
                        realm_id,
                        namespace_client_id,
                        payload,
                        fallback_role_id,
                        dry_run,
                        tx,
                    )
                    .await?;
                    return Ok(HarborImportResourceResult {
                        original_id: Some(original_name),
                        renamed_to: Some(renamed),
                        ..result
                    });
                }
                ConflictPolicy::Overwrite => {
                    if dry_run {
                        return Ok(HarborImportResourceResult {
                            key: self.key().to_string(),
                            status: "validated".to_string(),
                            created: 0,
                            updated: 1,
                            errors: Vec::new(),
                            original_id: Some(payload.name),
                            renamed_to: None,
                        });
                    }

                    let mut role = existing;
                    role.description = normalize_description(payload.description.clone());
                    if let Some(tx) = tx {
                        sync_role_permissions(
                            &*self.rbac_repo,
                            &role.id,
                            &payload.permissions,
                            Some(&mut *tx),
                        )
                        .await?;
                        self.rbac_repo.update_role(&role, Some(&mut *tx)).await?;
                    } else {
                        sync_role_permissions(
                            &*self.rbac_repo,
                            &role.id,
                            &payload.permissions,
                            None,
                        )
                        .await?;
                        self.rbac_repo.update_role(&role, None).await?;
                    }

                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "updated".to_string(),
                        created: 0,
                        updated: 1,
                        errors: Vec::new(),
                        original_id: Some(payload.name),
                        renamed_to: None,
                    });
                }
            }
        }

        import_new_role(
            &*self.rbac_repo,
            realm_id,
            namespace_client_id,
            payload,
            fallback_role_id,
            dry_run,
            tx,
        )
        .await
    }
}

async fn resolve_client_uuid(
    oidc_service: &OidcService,
    realm_id: Uuid,
    client_id: Option<&str>,
) -> Result<Option<Uuid>> {
    let Some(client_id) = client_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let client = oidc_service
        .find_client_by_client_id(&realm_id, client_id)
        .await?
        .ok_or_else(|| {
            Error::Validation(format!("Role references unknown client_id '{}'", client_id))
        })?;

    Ok(Some(client.id))
}

async fn find_role_in_namespace(
    repo: &dyn RbacRepository,
    realm_id: Uuid,
    client_id: Option<Uuid>,
    name: &str,
) -> Result<Option<Role>> {
    let mut page = 1;
    loop {
        let req = PageRequest {
            page,
            per_page: 200,
            q: Some(name.to_string()),
            ..PageRequest::default()
        };

        let response = match client_id {
            Some(client_id) => repo.list_client_roles(&realm_id, &client_id, &req).await?,
            None => repo.list_roles(&realm_id, &req).await?,
        };

        if let Some(role) = response.data.into_iter().find(|role| role.name == name) {
            return Ok(Some(role));
        }

        if response.meta.page >= response.meta.total_pages {
            break;
        }
        page += 1;
    }

    Ok(None)
}

async fn resolve_available_role_name(
    repo: &dyn RbacRepository,
    realm_id: Uuid,
    client_id: Option<Uuid>,
    base: &str,
) -> Result<String> {
    for idx in 1..=1000 {
        let candidate = format!("{}-{}", base, idx);
        if find_role_in_namespace(repo, realm_id, client_id, &candidate)
            .await?
            .is_none()
        {
            return Ok(candidate);
        }
    }

    Err(Error::Validation(
        "Unable to generate unique role name".to_string(),
    ))
}

async fn sync_role_permissions(
    repo: &dyn RbacRepository,
    role_id: &Uuid,
    desired: &[String],
    tx: Option<&mut dyn Transaction>,
) -> Result<()> {
    let current = repo.get_permissions_for_role(role_id).await?;
    let current_set = current
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
    let desired_set = desired
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>();

    let to_add = desired_set
        .difference(&current_set)
        .cloned()
        .collect::<Vec<_>>();
    let to_remove = current_set
        .difference(&desired_set)
        .cloned()
        .collect::<Vec<_>>();

    if let Some(tx) = tx {
        if !to_add.is_empty() {
            repo.bulk_update_permissions(role_id, to_add, "add", Some(&mut *tx))
                .await?;
        }
        if !to_remove.is_empty() {
            repo.bulk_update_permissions(role_id, to_remove, "remove", Some(&mut *tx))
                .await?;
        }
    } else {
        if !to_add.is_empty() {
            repo.bulk_update_permissions(role_id, to_add, "add", None)
                .await?;
        }
        if !to_remove.is_empty() {
            repo.bulk_update_permissions(role_id, to_remove, "remove", None)
                .await?;
        }
    }

    Ok(())
}

async fn import_new_role(
    repo: &dyn RbacRepository,
    realm_id: Uuid,
    client_id: Option<Uuid>,
    payload: HarborRolePayload,
    fallback_role_id: Uuid,
    dry_run: bool,
    tx: Option<&mut dyn Transaction>,
) -> Result<HarborImportResourceResult> {
    if dry_run {
        return Ok(HarborImportResourceResult {
            key: "role".to_string(),
            status: "validated".to_string(),
            created: 1,
            updated: 0,
            errors: Vec::new(),
            original_id: Some(payload.name),
            renamed_to: None,
        });
    }

    let role = Role {
        id: payload
            .role_id
            .as_deref()
            .and_then(|value| Uuid::parse_str(value).ok())
            .unwrap_or(fallback_role_id),
        realm_id,
        client_id,
        name: payload.name.clone(),
        description: normalize_description(payload.description),
    };

    if let Some(tx) = tx {
        repo.create_role(&role, Some(&mut *tx)).await?;
        sync_role_permissions(repo, &role.id, &payload.permissions, Some(&mut *tx)).await?;
    } else {
        repo.create_role(&role, None).await?;
        sync_role_permissions(repo, &role.id, &payload.permissions, None).await?;
    }

    Ok(HarborImportResourceResult {
        key: "role".to_string(),
        status: "created".to_string(),
        created: 1,
        updated: 0,
        errors: Vec::new(),
        original_id: Some(payload.name),
        renamed_to: None,
    })
}

fn normalize_description(description: Option<String>) -> Option<String> {
    description.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
