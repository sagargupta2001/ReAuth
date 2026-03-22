use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::application::oidc_service::OidcService;
use crate::domain::pagination::PageRequest;
use crate::domain::role::Role;
use crate::domain::user::User;
use crate::error::{Error, Result};
use crate::ports::rbac_repository::RbacRepository;
use crate::ports::transaction_manager::Transaction;
use crate::ports::user_repository::UserRepository;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const REDACTED_CREDENTIAL: &str = "${REDACTED}";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct HarborUserRoleRef {
    name: String,
    #[serde(default)]
    client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborUserPayload {
    #[serde(default)]
    user_id: Option<String>,
    username: String,
    #[serde(default)]
    hashed_password: Option<String>,
    #[serde(default)]
    direct_roles: Vec<HarborUserRoleRef>,
}

struct UserImportContext<'a> {
    user_repo: &'a dyn UserRepository,
    rbac_repo: &'a dyn RbacRepository,
    realm_id: Uuid,
    fallback_user_id: Uuid,
    dry_run: bool,
}

pub struct UserHarborProvider {
    user_repo: Arc<dyn UserRepository>,
    rbac_repo: Arc<dyn RbacRepository>,
    oidc_service: Arc<OidcService>,
}

impl UserHarborProvider {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        rbac_repo: Arc<dyn RbacRepository>,
        oidc_service: Arc<OidcService>,
    ) -> Self {
        Self {
            user_repo,
            rbac_repo,
            oidc_service,
        }
    }
}

#[async_trait]
impl HarborProvider for UserHarborProvider {
    fn key(&self) -> &'static str {
        "user"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        if !resource.assets.is_empty() {
            return Err(Error::Validation(
                "User bundles must not include assets".to_string(),
            ));
        }

        let payload: HarborUserPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid user bundle payload: {}", err)))?;

        if payload.username.trim().is_empty() {
            return Err(Error::Validation("Username is required".to_string()));
        }

        for role in payload.direct_roles {
            if role.name.trim().is_empty() {
                return Err(Error::Validation("User role name is required".to_string()));
            }
        }

        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        let user_id = match scope {
            HarborScope::User { user_id } => *user_id,
            _ => {
                return Err(Error::Validation(
                    "User export requires user scope".to_string(),
                ))
            }
        };

        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(Error::UserNotFound)?;
        if user.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "User belongs to different realm".to_string(),
            ));
        }

        let direct_role_ids = self
            .rbac_repo
            .find_direct_role_ids_for_user(&user.id)
            .await?;
        let mut direct_roles = Vec::new();
        for role_id in direct_role_ids {
            let role = self
                .rbac_repo
                .find_role_by_id(&role_id)
                .await?
                .ok_or_else(|| Error::NotFound("Role not found".to_string()))?;
            let client_id = match role.client_id {
                Some(client_uuid) => {
                    Some(self.oidc_service.get_client(client_uuid).await?.client_id)
                }
                None => None,
            };
            direct_roles.push(HarborUserRoleRef {
                name: role.name,
                client_id,
            });
        }
        direct_roles.sort_by(|a, b| {
            a.client_id
                .cmp(&b.client_id)
                .then_with(|| a.name.cmp(&b.name))
        });

        let hashed_password = match policy {
            ExportPolicy::IncludeSecrets => Some(user.hashed_password.clone()),
            ExportPolicy::Redact => Some(REDACTED_CREDENTIAL.to_string()),
        };

        let payload = HarborUserPayload {
            user_id: Some(user.id.to_string()),
            username: user.username,
            hashed_password,
            direct_roles,
        };

        Ok(HarborResourceBundle {
            key: self.key().to_string(),
            data: to_value(payload)
                .map_err(|err| Error::System(format!("Failed to serialize user: {}", err)))?,
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
        let scoped_user_id = match scope {
            HarborScope::User { user_id } => *user_id,
            _ => {
                return Err(Error::Validation(
                    "User import requires user scope".to_string(),
                ))
            }
        };

        let mut payload: HarborUserPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid user bundle payload: {}", err)))?;

        if let Some(payload_id) = payload.user_id.as_deref() {
            if Uuid::parse_str(payload_id).ok() != Some(scoped_user_id) {
                return Err(Error::Validation(
                    "User bundle id does not match import scope".to_string(),
                ));
            }
        }

        let existing = self
            .user_repo
            .find_by_username(&realm_id, &payload.username)
            .await?;
        let desired_role_ids = resolve_role_refs(
            &*self.rbac_repo,
            &self.oidc_service,
            realm_id,
            &payload.direct_roles,
        )
        .await?;

        if existing.is_none() && credentials_redacted(payload.hashed_password.as_deref()) {
            return Err(Error::Validation(
                "User bundle credentials are redacted; use include_secrets to create users in a new realm".to_string(),
            ));
        }

        if let Some(existing) = existing {
            match conflict_policy {
                ConflictPolicy::Skip => {
                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "skipped".to_string(),
                        created: 0,
                        updated: 0,
                        errors: Vec::new(),
                        original_id: Some(payload.username),
                        renamed_to: None,
                    });
                }
                ConflictPolicy::Rename => {
                    if credentials_redacted(payload.hashed_password.as_deref()) {
                        return Err(Error::Validation(
                            "User rename import requires credential material; redacted credentials cannot create a duplicate user".to_string(),
                        ));
                    }

                    let original_username = payload.username.clone();
                    let renamed =
                        resolve_available_username(&*self.user_repo, realm_id, &payload.username)
                            .await?;
                    payload.username = renamed.clone();
                    let result = import_new_user(
                        UserImportContext {
                            user_repo: &*self.user_repo,
                            rbac_repo: &*self.rbac_repo,
                            realm_id,
                            fallback_user_id: scoped_user_id,
                            dry_run,
                        },
                        payload,
                        desired_role_ids,
                        tx,
                    )
                    .await?;
                    return Ok(HarborImportResourceResult {
                        original_id: Some(original_username),
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
                            original_id: Some(payload.username),
                            renamed_to: None,
                        });
                    }

                    let mut user = existing;
                    if let Some(hash) = payload.hashed_password.as_deref() {
                        if !credentials_redacted(Some(hash)) && !hash.trim().is_empty() {
                            user.hashed_password = hash.to_string();
                        }
                    }

                    if let Some(tx) = tx {
                        self.user_repo.update(&user, Some(&mut *tx)).await?;
                        sync_direct_roles(
                            &*self.rbac_repo,
                            user.id,
                            &desired_role_ids,
                            Some(&mut *tx),
                        )
                        .await?;
                    } else {
                        self.user_repo.update(&user, None).await?;
                        sync_direct_roles(&*self.rbac_repo, user.id, &desired_role_ids, None)
                            .await?;
                    }

                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "updated".to_string(),
                        created: 0,
                        updated: 1,
                        errors: Vec::new(),
                        original_id: Some(payload.username),
                        renamed_to: None,
                    });
                }
            }
        }

        import_new_user(
            UserImportContext {
                user_repo: &*self.user_repo,
                rbac_repo: &*self.rbac_repo,
                realm_id,
                fallback_user_id: scoped_user_id,
                dry_run,
            },
            payload,
            desired_role_ids,
            tx,
        )
        .await
    }
}

async fn resolve_available_username(
    user_repo: &dyn UserRepository,
    realm_id: Uuid,
    base: &str,
) -> Result<String> {
    for idx in 1..=1000 {
        let candidate = format!("{}-{}", base, idx);
        if user_repo
            .find_by_username(&realm_id, &candidate)
            .await?
            .is_none()
        {
            return Ok(candidate);
        }
    }

    Err(Error::Validation(
        "Unable to generate unique username".to_string(),
    ))
}

async fn resolve_role_refs(
    repo: &dyn RbacRepository,
    oidc_service: &OidcService,
    realm_id: Uuid,
    refs: &[HarborUserRoleRef],
) -> Result<Vec<Uuid>> {
    let mut ids = Vec::new();
    for role_ref in refs {
        let role = find_role_ref(repo, oidc_service, realm_id, role_ref).await?;
        ids.push(role.id);
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

async fn find_role_ref(
    repo: &dyn RbacRepository,
    oidc_service: &OidcService,
    realm_id: Uuid,
    role_ref: &HarborUserRoleRef,
) -> Result<Role> {
    if let Some(client_id) = role_ref.client_id.as_deref() {
        let client = oidc_service
            .find_client_by_client_id(&realm_id, client_id)
            .await?
            .ok_or_else(|| {
                Error::Validation(format!(
                    "User role references unknown client_id '{}'",
                    client_id
                ))
            })?;

        let mut page = 1;
        loop {
            let response = repo
                .list_client_roles(
                    &realm_id,
                    &client.id,
                    &PageRequest {
                        page,
                        per_page: 200,
                        q: Some(role_ref.name.clone()),
                        ..PageRequest::default()
                    },
                )
                .await?;

            if let Some(role) = response
                .data
                .into_iter()
                .find(|role| role.name == role_ref.name)
            {
                return Ok(role);
            }

            if response.meta.page >= response.meta.total_pages {
                break;
            }
            page += 1;
        }

        return Err(Error::Validation(format!(
            "User role '{}' not found for client '{}'",
            role_ref.name, client_id
        )));
    }

    repo.find_role_by_name(&realm_id, &role_ref.name)
        .await?
        .ok_or_else(|| Error::Validation(format!("User role '{}' not found", role_ref.name)))
}

async fn sync_direct_roles(
    repo: &dyn RbacRepository,
    user_id: Uuid,
    desired_role_ids: &[Uuid],
    tx: Option<&mut dyn Transaction>,
) -> Result<()> {
    let current = repo.find_direct_role_ids_for_user(&user_id).await?;
    let current_set = current.into_iter().collect::<HashSet<_>>();
    let desired_set = desired_role_ids.iter().copied().collect::<HashSet<_>>();

    if let Some(tx) = tx {
        for role_id in current_set.difference(&desired_set) {
            repo.remove_role_from_user(&user_id, role_id, Some(&mut *tx))
                .await?;
        }
        for role_id in desired_set.difference(&current_set) {
            repo.assign_role_to_user(&user_id, role_id, Some(&mut *tx))
                .await?;
        }
    } else {
        for role_id in current_set.difference(&desired_set) {
            repo.remove_role_from_user(&user_id, role_id, None).await?;
        }
        for role_id in desired_set.difference(&current_set) {
            repo.assign_role_to_user(&user_id, role_id, None).await?;
        }
    }

    Ok(())
}

async fn import_new_user(
    ctx: UserImportContext<'_>,
    payload: HarborUserPayload,
    desired_role_ids: Vec<Uuid>,
    tx: Option<&mut dyn Transaction>,
) -> Result<HarborImportResourceResult> {
    if ctx.dry_run {
        return Ok(HarborImportResourceResult {
            key: "user".to_string(),
            status: "validated".to_string(),
            created: 1,
            updated: 0,
            errors: Vec::new(),
            original_id: Some(payload.username),
            renamed_to: None,
        });
    }

    let hashed_password = payload
        .hashed_password
        .clone()
        .filter(|value| !credentials_redacted(Some(value)));
    let Some(hashed_password) = hashed_password else {
        return Err(Error::Validation(
            "User bundle credentials are redacted; cannot create user".to_string(),
        ));
    };

    let candidate_id = payload
        .user_id
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or(ctx.fallback_user_id);
    let user_id = match ctx.user_repo.find_by_id(&candidate_id).await? {
        Some(existing) if existing.realm_id != ctx.realm_id => Uuid::new_v4(),
        Some(_) => candidate_id,
        None => candidate_id,
    };

    let user = User {
        id: user_id,
        realm_id: ctx.realm_id,
        username: payload.username.clone(),
        email: None,
        hashed_password,
    };

    if let Some(tx) = tx {
        ctx.user_repo.save(&user, Some(&mut *tx)).await?;
        sync_direct_roles(ctx.rbac_repo, user.id, &desired_role_ids, Some(&mut *tx)).await?;
    } else {
        ctx.user_repo.save(&user, None).await?;
        sync_direct_roles(ctx.rbac_repo, user.id, &desired_role_ids, None).await?;
    }

    Ok(HarborImportResourceResult {
        key: "user".to_string(),
        status: "created".to_string(),
        created: 1,
        updated: 0,
        errors: Vec::new(),
        original_id: Some(payload.username),
        renamed_to: None,
    })
}

fn credentials_redacted(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some(REDACTED_CREDENTIAL) | None | Some("")
    )
}
