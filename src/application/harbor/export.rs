use super::service::HarborService;
use crate::application::harbor::schema::*;
use crate::application::harbor::service::ImportProgress;
use crate::application::harbor::types::*;
use crate::domain::flow::models::FlowDraft;
use crate::domain::harbor_job::HarborJob;
use crate::domain::harbor_job_conflict::HarborJobConflict;
use crate::domain::pagination::PageRequest;
use crate::domain::role::Role;
use crate::domain::user::User;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use chrono::Utc;
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

use super::utils::*;
impl HarborService {
    pub async fn export_bundle(
        &self,
        realm_id: Uuid,
        source_realm: &str,
        scope: HarborScope,
        policy: ExportPolicy,
        selection: Option<Vec<String>>,
    ) -> Result<HarborBundle> {
        self.export_bundle_internal(realm_id, source_realm, scope, policy, selection, None, true)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn export_bundle_with_job(
        &self,
        realm_id: Uuid,
        source_realm: &str,
        scope: HarborScope,
        policy: ExportPolicy,
        selection: Option<Vec<String>>,
        job_id: Uuid,
        finalize_job: bool,
    ) -> Result<HarborBundle> {
        self.export_bundle_internal(
            realm_id,
            source_realm,
            scope,
            policy,
            selection,
            Some(job_id),
            finalize_job,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn export_bundle_internal(
        &self,
        realm_id: Uuid,
        source_realm: &str,
        scope: HarborScope,
        policy: ExportPolicy,
        selection: Option<Vec<String>>,
        job_override: Option<Uuid>,
        finalize_job: bool,
    ) -> Result<HarborBundle> {
        let job_id = match job_override {
            Some(id) => Some(id),
            None => {
                self.start_job(
                    realm_id,
                    super::service::HARBOR_JOB_TYPE_EXPORT,
                    &scope,
                    0,
                    false,
                    None,
                )
                .await
            }
        };

        let result = if let HarborScope::FullRealm = scope {
            self.export_full_bundle(realm_id, source_realm, policy, selection, job_id)
                .await
        } else {
            let provider_key = scope
                .provider_key()
                .ok_or_else(|| Error::Validation("Unsupported export scope".to_string()))?;
            let provider = self.registry.get(provider_key).ok_or_else(|| {
                Error::System(format!(
                    "Harbor provider not registered for scope: {}",
                    provider_key
                ))
            })?;

            let resource = provider.export(realm_id, &scope, policy).await?;
            if let Some(job_id) = job_id {
                self.try_update_job_total(job_id, 1).await;
                self.try_update_job_progress(job_id, 1, 0, 0).await;
            }

            Ok(HarborBundle {
                manifest: HarborManifest {
                    version: super::service::HARBOR_BUNDLE_VERSION.to_string(),
                    schema_version: super::service::HARBOR_SCHEMA_VERSION,
                    exported_at: Utc::now().to_rfc3339(),
                    source_realm: source_realm.to_string(),
                    export_type: scope.export_type(),
                    selection: None,
                },
                resources: vec![resource],
            })
        };

        match result {
            Ok(mut bundle) => {
                normalize_bundle_for_export(&mut bundle);
                if finalize_job {
                    if let Some(job_id) = job_id {
                        self.try_mark_completed(job_id, bundle.resources.len() as i64, 0, 0)
                            .await;
                    }
                }
                Ok(bundle)
            }
            Err(err) => {
                if let Some(job_id) = job_id {
                    self.try_mark_failed(job_id, &err).await;
                }
                Err(err)
            }
        }
    }

    pub(crate) async fn export_full_bundle(
        &self,
        realm_id: Uuid,
        source_realm: &str,
        policy: ExportPolicy,
        selection: Option<Vec<String>>,
        job_id: Option<Uuid>,
    ) -> Result<HarborBundle> {
        let selection = normalize_export_selection(selection)?;
        let mut resources = Vec::new();
        let mut total = 0usize;

        let mut themes = Vec::new();
        if selection.iter().any(|key| key == "theme") {
            themes = self.theme_service.list_themes(realm_id).await?;
            total += themes.len();
        }

        let mut clients = Vec::new();
        if selection.iter().any(|key| key == "client") {
            clients = self.list_all_clients(realm_id).await?;
            total += clients.len();
        }

        let mut roles = Vec::new();
        if selection.iter().any(|key| key == "role") {
            roles = self.list_all_roles(realm_id).await?;
            total += roles.len();
        }

        let mut users = Vec::new();
        if selection.iter().any(|key| key == "user") {
            users = self.list_all_users(realm_id).await?;
            total += users.len();
        }

        let mut flow_ids = Vec::new();
        if selection.iter().any(|key| key == "flow") {
            flow_ids = self.list_all_flow_ids_for_export(realm_id).await?;
            total += flow_ids.len();
        }

        if selection.iter().any(|key| key == "realm") {
            total += 1;
        }

        if let Some(job_id) = job_id {
            self.try_update_job_total(job_id, total as i64).await;
        }

        let mut processed = 0i64;

        if selection.iter().any(|key| key == "theme") {
            let provider = self
                .registry
                .get("theme")
                .ok_or_else(|| Error::Validation("Theme provider not registered".to_string()))?;
            for theme in themes {
                let scope = HarborScope::Theme { theme_id: theme.id };
                let resource = provider.export(realm_id, &scope, policy).await?;
                resources.push(resource);
                processed += 1;
                if let Some(job_id) = job_id {
                    self.try_update_job_progress(job_id, processed, 0, 0).await;
                }
            }
        }

        if selection.iter().any(|key| key == "client") {
            let provider = self
                .registry
                .get("client")
                .ok_or_else(|| Error::Validation("Client provider not registered".to_string()))?;
            for client in clients {
                let scope = HarborScope::Client {
                    client_id: client.client_id,
                };
                let resource = provider.export(realm_id, &scope, policy).await?;
                resources.push(resource);
                processed += 1;
                if let Some(job_id) = job_id {
                    self.try_update_job_progress(job_id, processed, 0, 0).await;
                }
            }
        }

        if selection.iter().any(|key| key == "role") {
            let provider = self
                .registry
                .get("role")
                .ok_or_else(|| Error::Validation("Role provider not registered".to_string()))?;
            for role in roles {
                let scope = HarborScope::Role { role_id: role.id };
                let resource = provider.export(realm_id, &scope, policy).await?;
                resources.push(resource);
                processed += 1;
                if let Some(job_id) = job_id {
                    self.try_update_job_progress(job_id, processed, 0, 0).await;
                }
            }
        }

        if selection.iter().any(|key| key == "user") {
            let provider = self
                .registry
                .get("user")
                .ok_or_else(|| Error::Validation("User provider not registered".to_string()))?;
            for user in users {
                let scope = HarborScope::User { user_id: user.id };
                let resource = provider.export(realm_id, &scope, policy).await?;
                resources.push(resource);
                processed += 1;
                if let Some(job_id) = job_id {
                    self.try_update_job_progress(job_id, processed, 0, 0).await;
                }
            }
        }

        if selection.iter().any(|key| key == "flow") {
            let provider = self
                .registry
                .get("flow")
                .ok_or_else(|| Error::Validation("Flow provider not registered".to_string()))?;
            for flow_id in flow_ids {
                let scope = HarborScope::Flow { flow_id };
                let resource = provider.export(realm_id, &scope, policy).await?;
                resources.push(resource);
                processed += 1;
                if let Some(job_id) = job_id {
                    self.try_update_job_progress(job_id, processed, 0, 0).await;
                }
            }
        }

        if selection.iter().any(|key| key == "realm") {
            let provider = self
                .registry
                .get("realm")
                .ok_or_else(|| Error::Validation("Realm provider not registered".to_string()))?;
            let resource = provider
                .export(realm_id, &HarborScope::FullRealm, policy)
                .await?;
            resources.push(resource);
            processed += 1;
            if let Some(job_id) = job_id {
                self.try_update_job_progress(job_id, processed, 0, 0).await;
            }
        }

        if resources.is_empty() {
            return Err(Error::Validation(
                "Full realm export selection produced no resources".to_string(),
            ));
        }

        Ok(HarborBundle {
            manifest: HarborManifest {
                version: super::service::HARBOR_BUNDLE_VERSION.to_string(),
                schema_version: super::service::HARBOR_SCHEMA_VERSION,
                exported_at: Utc::now().to_rfc3339(),
                source_realm: source_realm.to_string(),
                export_type: HarborScope::FullRealm.export_type(),
                selection: Some(selection),
            },
            resources,
        })
    }

    pub(crate) async fn list_all_clients(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<crate::domain::oidc::OidcClient>> {
        let mut clients = Vec::new();
        let mut page = 1;
        loop {
            let response = self
                .oidc_service
                .list_clients(
                    realm_id,
                    PageRequest {
                        page,
                        per_page: 200,
                        ..PageRequest::default()
                    },
                )
                .await?;
            clients.extend(response.data);
            if response.meta.page >= response.meta.total_pages {
                break;
            }
            page += 1;
        }
        Ok(clients)
    }

    pub(crate) async fn list_all_flow_drafts(&self, realm_id: Uuid) -> Result<Vec<FlowDraft>> {
        let mut drafts = Vec::new();
        let mut page = 1;
        loop {
            let response = self
                .flow_manager
                .list_drafts(
                    realm_id,
                    PageRequest {
                        page,
                        per_page: 200,
                        ..PageRequest::default()
                    },
                )
                .await?;
            drafts.extend(response.data);
            if response.meta.page >= response.meta.total_pages {
                break;
            }
            page += 1;
        }
        Ok(drafts)
    }

    pub(crate) async fn list_all_flow_ids_for_export(&self, realm_id: Uuid) -> Result<Vec<Uuid>> {
        let mut ids = std::collections::HashSet::new();
        let mut ordered = Vec::new();

        for flow in self.flow_service.list_flows(realm_id).await? {
            if ids.insert(flow.id) {
                ordered.push(flow.id);
            }
        }

        for draft in self.list_all_flow_drafts(realm_id).await? {
            if ids.insert(draft.id) {
                ordered.push(draft.id);
            }
        }

        Ok(ordered)
    }

    pub(crate) async fn list_all_roles(&self, realm_id: Uuid) -> Result<Vec<Role>> {
        let mut roles = Vec::new();
        let mut page = 1;
        loop {
            let response = self
                .rbac_service
                .list_roles(
                    realm_id,
                    PageRequest {
                        page,
                        per_page: 200,
                        ..PageRequest::default()
                    },
                )
                .await?;
            roles.extend(response.data);
            if response.meta.page >= response.meta.total_pages {
                break;
            }
            page += 1;
        }

        for client in self.list_all_clients(realm_id).await? {
            let mut client_page = 1;
            loop {
                let response = self
                    .rbac_service
                    .list_client_roles(
                        realm_id,
                        client.id,
                        PageRequest {
                            page: client_page,
                            per_page: 200,
                            ..PageRequest::default()
                        },
                    )
                    .await?;
                roles.extend(response.data);
                if response.meta.page >= response.meta.total_pages {
                    break;
                }
                client_page += 1;
            }
        }

        Ok(roles)
    }

    pub(crate) async fn list_all_users(&self, realm_id: Uuid) -> Result<Vec<User>> {
        let mut users = Vec::new();
        let mut page = 1;
        loop {
            let response = self
                .user_service
                .list_users(
                    realm_id,
                    PageRequest {
                        page,
                        per_page: 200,
                        ..PageRequest::default()
                    },
                    crate::domain::user::UserListFilters::default(),
                )
                .await?;
            users.extend(response.data);
            if response.meta.page >= response.meta.total_pages {
                break;
            }
            page += 1;
        }
        Ok(users)
    }
}

impl HarborService {
    pub async fn estimate_export_size(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        selection: Option<Vec<String>>,
    ) -> Result<i64> {
        match scope {
            HarborScope::FullRealm => {
                let selection = normalize_export_selection(selection)?;
                let mut total = 0i64;
                if selection.iter().any(|key| key == "theme") {
                    total += self.theme_service.list_themes(realm_id).await?.len() as i64;
                }
                if selection.iter().any(|key| key == "client") {
                    total += self.list_all_clients(realm_id).await?.len() as i64;
                }
                if selection.iter().any(|key| key == "role") {
                    total += self.list_all_roles(realm_id).await?.len() as i64;
                }
                if selection.iter().any(|key| key == "user") {
                    total += self.list_all_users(realm_id).await?.len() as i64;
                }
                if selection.iter().any(|key| key == "flow") {
                    total += self.list_all_flow_ids_for_export(realm_id).await?.len() as i64;
                }
                if selection.iter().any(|key| key == "realm") {
                    total += 1;
                }
                Ok(total)
            }
            _ => Ok(1),
        }
    }

    pub(crate) fn validate_bundle(&self, bundle: &HarborBundle, scope: &HarborScope) -> Result<()> {
        let bundle_value = serde_json::to_value(bundle)
            .map_err(|err| Error::System(format!("Failed to encode bundle: {}", err)))?;
        validate_bundle_schema(&bundle_value)?;

        if bundle.manifest.version != super::service::HARBOR_BUNDLE_VERSION {
            return Err(Error::Validation(format!(
                "Unsupported bundle version: {}",
                bundle.manifest.version
            )));
        }

        let exported_at = bundle.manifest.exported_at.trim();
        if exported_at.is_empty() {
            return Err(Error::Validation(
                "Manifest exported_at is required".to_string(),
            ));
        }
        chrono::DateTime::parse_from_rfc3339(exported_at)
            .map_err(|_| Error::Validation("Manifest exported_at must be RFC3339".to_string()))?;

        if bundle.manifest.source_realm.trim().is_empty() {
            return Err(Error::Validation(
                "Manifest source_realm is required".to_string(),
            ));
        }

        if let Some(selection) = bundle.manifest.selection.as_ref() {
            normalize_export_selection(Some(selection.clone()))?;
        }

        if bundle.resources.is_empty() {
            return Err(Error::Validation(
                "Bundle contains no resources".to_string(),
            ));
        }

        if scope.export_type() != bundle.manifest.export_type {
            return Err(Error::Validation(
                "Bundle export type does not match import scope".to_string(),
            ));
        }

        let mut keys = std::collections::HashSet::new();
        for resource in &bundle.resources {
            if resource.key.trim().is_empty() {
                return Err(Error::Validation("Resource key is required".to_string()));
            }
            if !matches!(scope, HarborScope::FullRealm) && !keys.insert(resource.key.as_str()) {
                return Err(Error::Validation(
                    "Duplicate resource key in bundle".to_string(),
                ));
            }
            let resource_value = serde_json::to_value(resource)
                .map_err(|err| Error::System(format!("Failed to encode resource: {}", err)))?;
            validate_resource_schema(&resource.key, &resource_value)?;
        }

        for resource in &bundle.resources {
            let provider = self.registry.get(&resource.key).ok_or_else(|| {
                Error::Validation(format!("No provider registered for {}", resource.key))
            })?;
            provider.validate(resource)?;
        }

        Ok(())
    }

    pub fn validate_bundle_for_scope(
        &self,
        bundle: &HarborBundle,
        scope: &HarborScope,
    ) -> Result<()> {
        self.validate_bundle(bundle, scope)
    }

    pub(crate) async fn import_full_bundle(
        &self,
        realm_id: Uuid,
        bundle: HarborBundle,
        conflict_policy: ConflictPolicy,
        job_id: Option<Uuid>,
        persist_job_updates: bool,
        mut tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResult> {
        let mut results = Vec::new();
        let mut warnings = Vec::new();
        let mut client_id_map = std::collections::HashMap::new();
        let mut flow_id_map = std::collections::HashMap::new();
        let mut role_ref_map = std::collections::HashMap::new();
        let mut progress = ImportProgress {
            processed: 0,
            created_total: 0,
            updated_total: 0,
        };

        let client_provider = match self.registry.get("client") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("Client provider not registered".to_string());
                None
            }
        };

        let flow_provider = match self.registry.get("flow") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("Flow provider not registered".to_string());
                None
            }
        };

        let role_provider = match self.registry.get("role") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("Role provider not registered".to_string());
                None
            }
        };

        let user_provider = match self.registry.get("user") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("User provider not registered".to_string());
                None
            }
        };

        let theme_provider = match self.registry.get("theme") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("Theme provider not registered".to_string());
                None
            }
        };

        let realm_provider = match self.registry.get("realm") {
            Some(provider) => Some(provider),
            None => {
                warnings.push("Realm provider not registered".to_string());
                None
            }
        };

        let mut theme_ids_by_name = HashMap::new();
        let mut theme_cache_by_name = HashMap::new();
        if theme_provider.is_some() {
            for theme in self.theme_service.list_themes(realm_id).await? {
                theme_ids_by_name.insert(theme.name.clone(), theme.id);
                theme_cache_by_name.insert(theme.name.clone(), theme);
            }
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "client") {
            let Some(provider) = client_provider.as_ref() else {
                continue;
            };

            let client_id = resource
                .data
                .get("client_id")
                .and_then(|value| value.as_str())
                .ok_or_else(|| Error::Validation("Client bundle missing client_id".to_string()))?;

            let scope = HarborScope::Client {
                client_id: client_id.to_string(),
            };

            let result = provider
                .import(
                    realm_id,
                    &scope,
                    resource,
                    conflict_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            if let (Some(original), Some(renamed)) =
                (result.original_id.clone(), result.renamed_to.clone())
            {
                client_id_map.insert(original, renamed);
            }

            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "role") {
            let Some(provider) = role_provider.as_ref() else {
                continue;
            };

            let mut resource = resource.clone();
            if !client_id_map.is_empty() {
                rewrite_reference_ids(&mut resource.data, "client_id", &client_id_map);
            }

            let role_id = resource
                .data
                .get("role_id")
                .and_then(|value| value.as_str())
                .and_then(|value| Uuid::parse_str(value).ok())
                .unwrap_or_else(Uuid::new_v4);

            let scope = HarborScope::Role { role_id };

            let result = provider
                .import(
                    realm_id,
                    &scope,
                    &resource,
                    conflict_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            let role_name = resource
                .data
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let role_client_id = resource
                .data
                .get("client_id")
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            let role_ref = encode_role_ref(&role_name, role_client_id.as_deref());

            if let Some(renamed) = result.renamed_to.clone() {
                role_ref_map.insert(
                    role_ref,
                    encode_role_ref(&renamed, role_client_id.as_deref()),
                );
            }

            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "user") {
            let Some(provider) = user_provider.as_ref() else {
                continue;
            };

            let mut resource = resource.clone();
            if !client_id_map.is_empty() {
                rewrite_user_role_client_ids(&mut resource.data, &client_id_map);
            }
            if !role_ref_map.is_empty() {
                rewrite_user_role_refs(&mut resource.data, &role_ref_map);
            }

            let user_id = resource
                .data
                .get("user_id")
                .and_then(|value| value.as_str())
                .and_then(|value| Uuid::parse_str(value).ok())
                .unwrap_or_else(Uuid::new_v4);

            let scope = HarborScope::User { user_id };
            let result = provider
                .import(
                    realm_id,
                    &scope,
                    &resource,
                    conflict_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "flow") {
            let Some(provider) = flow_provider.as_ref() else {
                continue;
            };

            let mut resource = resource.clone();
            if !client_id_map.is_empty() {
                rewrite_reference_ids(&mut resource.data, "client_id", &client_id_map);
            }

            let flow_id = resource
                .data
                .get("flow_id")
                .and_then(|value| value.as_str())
                .and_then(|value| Uuid::parse_str(value).ok())
                .unwrap_or_else(Uuid::new_v4);

            let scope = HarborScope::Flow { flow_id };

            let result = provider
                .import(
                    realm_id,
                    &scope,
                    &resource,
                    conflict_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            if let (Some(original), Some(renamed)) =
                (result.original_id.clone(), result.renamed_to.clone())
            {
                flow_id_map.insert(original, renamed);
            }

            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "realm") {
            let Some(provider) = realm_provider.as_ref() else {
                continue;
            };

            let mut resource = resource.clone();
            if !flow_id_map.is_empty() {
                rewrite_realm_flow_bindings(&mut resource.data, &flow_id_map);
            }

            let result = provider
                .import(
                    realm_id,
                    &HarborScope::FullRealm,
                    &resource,
                    conflict_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        for resource in bundle.resources.iter().filter(|r| r.key == "theme") {
            let Some(provider) = theme_provider.as_ref() else {
                continue;
            };

            let mut resource = resource.clone();
            if !client_id_map.is_empty() {
                if let Some(meta) = resource.meta.as_mut() {
                    rewrite_reference_ids(meta, "client_id", &client_id_map);
                }
            }

            let meta = parse_theme_meta(&resource)?;
            let theme_info = meta.theme.ok_or_else(|| {
                Error::Validation("Theme bundle missing theme metadata".to_string())
            })?;

            let mut theme_id = None;
            let mut theme_created = false;
            let mut theme_name = theme_info.name.clone();
            if let Some(existing) = theme_cache_by_name.get(&theme_info.name) {
                match conflict_policy {
                    ConflictPolicy::Skip => {
                        let result = HarborImportResourceResult {
                            key: "theme".to_string(),
                            status: "skipped".to_string(),
                            created: 0,
                            updated: 0,
                            errors: Vec::new(),
                            original_id: Some(theme_info.name.clone()),
                            renamed_to: None,
                        };
                        self.record_import_progress(
                            job_id,
                            persist_job_updates,
                            &mut progress,
                            &result,
                            conflict_policy,
                        )
                        .await;
                        results.push(result);
                        continue;
                    }
                    ConflictPolicy::Overwrite => {
                        theme_id = Some(existing.id);
                        if theme_info.description != existing.description {
                            let description = theme_info.description.clone().unwrap_or_default();
                            let _ = self
                                .theme_service
                                .update_theme_with_tx(
                                    realm_id,
                                    existing.id,
                                    None,
                                    Some(description),
                                    tx.as_deref_mut(),
                                )
                                .await?;
                        }
                    }
                    ConflictPolicy::Rename => {
                        theme_name =
                            resolve_available_theme_name(&theme_ids_by_name, &theme_info.name)?;
                        theme_created = true;
                        warnings.push(format!(
                            "Theme '{}' renamed to '{}' during import",
                            theme_info.name, theme_name
                        ));
                    }
                }
            } else {
                theme_created = true;
            }

            if theme_created {
                let created = if let Some(tx_ref) = tx.as_deref_mut() {
                    self.theme_service
                        .create_theme_with_tx(
                            realm_id,
                            theme_name.clone(),
                            theme_info.description.clone(),
                            tx_ref,
                        )
                        .await?
                } else {
                    self.theme_service
                        .create_theme(realm_id, theme_name.clone(), theme_info.description.clone())
                        .await?
                };
                theme_id = Some(created.id);
                theme_ids_by_name.insert(created.name.clone(), created.id);
                theme_cache_by_name.insert(created.name.clone(), created);
            }

            let Some(theme_id) = theme_id else {
                return Err(Error::System(
                    "Theme import failed to resolve theme id".to_string(),
                ));
            };

            let effective_policy = if matches!(conflict_policy, ConflictPolicy::Rename) {
                ConflictPolicy::Overwrite
            } else {
                conflict_policy
            };

            let scope = HarborScope::Theme { theme_id };
            let mut result = provider
                .import(
                    realm_id,
                    &scope,
                    &resource,
                    effective_policy,
                    false,
                    tx.as_deref_mut(),
                )
                .await?;

            if theme_created {
                result.created += 1;
            }

            let bindings = meta.bindings.unwrap_or_default();
            let mut binding_created = 0u32;
            let mut binding_updated = 0u32;

            if bindings.default {
                let existing_default = if let Some(tx_ref) = tx.as_deref_mut() {
                    self.theme_service
                        .resolve_binding_with_tx(realm_id, None, Some(tx_ref))
                        .await?
                } else {
                    self.theme_service.resolve_binding(realm_id, None).await?
                };
                if existing_default.is_some() {
                    binding_updated += 1;
                } else {
                    binding_created += 1;
                }
            }

            for binding in &bindings.clients {
                let existing = if let Some(tx_ref) = tx.as_deref_mut() {
                    self.theme_service
                        .get_binding_for_client_with_tx(realm_id, &binding.client_id, Some(tx_ref))
                        .await?
                } else {
                    self.theme_service
                        .get_binding_for_client(realm_id, &binding.client_id)
                        .await?
                };
                if existing.is_some() {
                    binding_updated += 1;
                } else {
                    binding_created += 1;
                }
            }

            let version = if let Some(tx_ref) = tx.as_deref_mut() {
                self.theme_service
                    .publish_theme_with_tx(realm_id, theme_id, Some(tx_ref))
                    .await?
            } else {
                self.theme_service.publish_theme(realm_id, theme_id).await?
            };

            if bindings.default {
                self.theme_service
                    .activate_version_with_tx(realm_id, theme_id, version.id, tx.as_deref_mut())
                    .await?;
            }

            for binding in bindings.clients {
                self.theme_service
                    .upsert_client_binding_with_tx(
                        realm_id,
                        binding.client_id,
                        theme_id,
                        version.id,
                        tx.as_deref_mut(),
                    )
                    .await?;
            }

            result.created += binding_created;
            result.updated += binding_updated;
            self.record_import_progress(
                job_id,
                persist_job_updates,
                &mut progress,
                &result,
                conflict_policy,
            )
            .await;
            results.push(result);
        }

        Ok(HarborImportResult {
            dry_run: false,
            resources: results,
            warnings,
        })
    }

    pub(crate) async fn record_import_progress(
        &self,
        job_id: Option<Uuid>,
        persist_job_updates: bool,
        progress: &mut ImportProgress,
        result: &HarborImportResourceResult,
        conflict_policy: ConflictPolicy,
    ) {
        progress.processed += 1;
        progress.created_total += result.created as i64;
        progress.updated_total += result.updated as i64;
        if persist_job_updates {
            if let Some(job_id) = job_id {
                self.try_update_job_progress(
                    job_id,
                    progress.processed,
                    progress.created_total,
                    progress.updated_total,
                )
                .await;
                self.try_record_conflict(job_id, result, conflict_policy)
                    .await;
            }
        }
    }

    pub(crate) async fn start_job(
        &self,
        realm_id: Uuid,
        job_type: &str,
        scope: &HarborScope,
        total_resources: i64,
        dry_run: bool,
        conflict_policy: Option<ConflictPolicy>,
    ) -> Option<Uuid> {
        match self
            .create_job(
                realm_id,
                job_type,
                scope,
                total_resources,
                dry_run,
                conflict_policy,
            )
            .await
        {
            Ok(job_id) => Some(job_id),
            Err(err) => {
                warn!("Failed to create harbor job: {}", err);
                None
            }
        }
    }

    pub async fn create_job(
        &self,
        realm_id: Uuid,
        job_type: &str,
        scope: &HarborScope,
        total_resources: i64,
        dry_run: bool,
        conflict_policy: Option<ConflictPolicy>,
    ) -> Result<Uuid> {
        let now = Utc::now();
        let job_id = Uuid::new_v4();
        let job = HarborJob {
            id: job_id,
            realm_id,
            job_type: job_type.to_string(),
            status: super::service::HARBOR_JOB_STATUS_IN_PROGRESS.to_string(),
            scope: scope_label(scope).to_string(),
            total_resources,
            processed_resources: 0,
            created_count: 0,
            updated_count: 0,
            dry_run,
            conflict_policy: conflict_policy
                .map(conflict_policy_label)
                .map(|s| s.to_string()),
            artifact_path: None,
            artifact_filename: None,
            artifact_content_type: None,
            error_message: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        self.job_repo.create(&job).await?;

        Ok(job_id)
    }

    pub(crate) async fn try_update_job_total(&self, job_id: Uuid, total: i64) {
        if let Err(err) = self.job_repo.update_total(&job_id, total).await {
            warn!("Failed to update harbor job total: {}", err);
        }
    }

    pub(crate) async fn try_update_job_progress(
        &self,
        job_id: Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) {
        if let Err(err) = self
            .job_repo
            .update_progress(&job_id, processed, created, updated)
            .await
        {
            warn!("Failed to update harbor job progress: {}", err);
        }
    }

    pub async fn set_job_artifact(
        &self,
        job_id: Uuid,
        path: &str,
        filename: &str,
        content_type: &str,
    ) -> Result<()> {
        self.job_repo
            .update_artifact(&job_id, path, filename, content_type)
            .await
    }

    pub(crate) async fn try_mark_completed(
        &self,
        job_id: Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) {
        if let Err(err) = self
            .job_repo
            .mark_completed(&job_id, processed, created, updated)
            .await
        {
            warn!("Failed to mark harbor job completed: {}", err);
        }
    }

    pub(crate) async fn try_mark_failed(&self, job_id: Uuid, err: &Error) {
        if let Err(err) = self.job_repo.mark_failed(&job_id, &err.to_string()).await {
            warn!("Failed to mark harbor job failed: {}", err);
        }
    }

    pub async fn list_jobs(&self, realm_id: Uuid, limit: i64) -> Result<Vec<HarborJob>> {
        self.job_repo.list_recent(&realm_id, limit).await
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<Option<HarborJob>> {
        self.job_repo.find_by_id(&job_id).await
    }

    pub fn spawn_job(&self, task: futures::future::BoxFuture<'static, ()>) {
        self.job_runner.spawn(task);
    }

    pub async fn mark_job_completed(
        &self,
        job_id: Uuid,
        processed: i64,
        created: i64,
        updated: i64,
    ) -> Result<()> {
        self.job_repo
            .mark_completed(&job_id, processed, created, updated)
            .await
    }

    pub async fn mark_job_failed(&self, job_id: Uuid, message: &str) -> Result<()> {
        self.job_repo.mark_failed(&job_id, message).await
    }

    pub async fn list_job_conflicts(&self, job_id: Uuid) -> Result<Vec<HarborJobConflict>> {
        self.conflict_repo.list_by_job(&job_id).await
    }

    pub(crate) async fn try_record_conflict(
        &self,
        job_id: Uuid,
        result: &HarborImportResourceResult,
        conflict_policy: ConflictPolicy,
    ) {
        let action = if result.renamed_to.is_some() {
            Some("renamed")
        } else if result.status == "skipped" {
            Some("skipped")
        } else {
            None
        };

        let Some(action) = action else {
            return;
        };

        let conflict = HarborJobConflict {
            id: Uuid::new_v4(),
            job_id,
            resource_key: result.key.clone(),
            action: action.to_string(),
            policy: conflict_policy_label(conflict_policy).to_string(),
            original_id: result.original_id.clone(),
            resolved_id: result.renamed_to.clone(),
            message: None,
            created_at: Utc::now(),
        };

        if let Err(err) = self.conflict_repo.create(&conflict).await {
            warn!("Failed to record harbor conflict: {}", err);
        }
    }
}
