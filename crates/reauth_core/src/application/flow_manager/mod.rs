#![allow(clippy::needless_option_as_deref)]

pub mod templates;

use crate::application::flow_manager::templates::FlowTemplates;
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::compiler::flow_compiler::FlowCompiler;
use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use crate::ports::flow_repository::FlowRepository;
use crate::{
    domain::pagination::{PageRequest, PageResponse},
    error::{Error, Result},
    ports::transaction_manager::Transaction,
    ports::{flow_store::FlowStore, realm_repository::RealmRepository},
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDraftRequest {
    pub name: String,
    pub description: Option<String>,
    pub flow_type: String,
}

#[derive(Deserialize)]
pub struct UpdateDraftRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub graph_json: Option<serde_json::Value>,
}

pub struct FlowManager {
    flow_store: Arc<dyn FlowStore>,
    flow_repo: Arc<dyn FlowRepository>,
    realm_repo: Arc<dyn RealmRepository>,
    runtime_registry: Arc<RuntimeRegistry>,
}

impl FlowManager {
    pub fn new(
        flow_store: Arc<dyn FlowStore>,
        flow_repo: Arc<dyn FlowRepository>,
        realm_repo: Arc<dyn RealmRepository>,
        runtime_registry: Arc<RuntimeRegistry>,
    ) -> Self {
        Self {
            flow_store,
            flow_repo,
            realm_repo,
            runtime_registry,
        }
    }

    pub async fn list_drafts(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<FlowDraft>> {
        self.flow_store.list_drafts(&realm_id, &req).await
    }

    pub async fn create_draft(&self, realm_id: Uuid, req: CreateDraftRequest) -> Result<FlowDraft> {
        // Check duplicate name
        // (Assuming list filter can be used or we catch DB error. For now catch DB error).

        let draft = FlowDraft {
            id: Uuid::new_v4(),
            realm_id,
            name: req.name,
            description: req.description,
            graph_json: "{}".to_string(),
            flow_type: req.flow_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.flow_store.create_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn get_draft(&self, id: Uuid) -> Result<FlowDraft> {
        // 1. Try to find an existing active draft
        if let Some(draft) = self.flow_store.get_draft_by_id(&id).await? {
            return Ok(draft);
        }

        // 2. If not found, check if there is a Published Version we can restore from
        // (This implements "Lazy Draft Creation" / "Auto-Fork")
        if let Some(latest_version) = self.flow_store.get_latest_version(&id).await? {
            // Fetch flow metadata to get the name/type
            let flow_meta = self
                .flow_repo
                .find_flow_by_id(&id)
                .await?
                .ok_or(Error::FlowNotFound(id.to_string()))?;

            let new_draft = FlowDraft {
                id, // Same ID as the flow
                realm_id: flow_meta.realm_id,
                name: flow_meta.name,
                description: flow_meta.description,
                graph_json: latest_version.graph_json, // RESTORE UI FROM VERSION
                flow_type: flow_meta.r#type,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Save this restored draft so next time Step 1 succeeds
            self.flow_store.create_draft(&new_draft).await?;
            return Ok(new_draft);
        }

        // 3. Emergency Fallback (For your currently broken flow)
        // If no draft AND no version exists (or version has no graph_json yet),
        // but the Flow ID is valid, seed a blank graph to stop the 500 error.
        if let Some(flow_meta) = self.flow_repo.find_flow_by_id(&id).await? {
            let blank_json = Self::generate_default_graph(&flow_meta.r#type);

            let recovery_draft = FlowDraft {
                id,
                realm_id: flow_meta.realm_id,
                name: flow_meta.name,
                description: flow_meta.description,
                graph_json: blank_json,
                flow_type: flow_meta.r#type,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.flow_store.create_draft(&recovery_draft).await?;
            return Ok(recovery_draft);
        }

        Err(Error::FlowNotFound(id.to_string()))
    }

    pub async fn update_draft(&self, id: Uuid, req: UpdateDraftRequest) -> Result<FlowDraft> {
        let mut draft = self.get_draft(id).await?;

        if let Some(n) = req.name {
            draft.name = n;
        }
        if let Some(d) = req.description {
            draft.description = Some(d);
        }
        if let Some(json) = req.graph_json {
            draft.graph_json = json.to_string();
        }
        draft.updated_at = Utc::now();

        self.flow_store.update_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn list_all_drafts(&self, realm_id: Uuid) -> Result<Vec<FlowDraft>> {
        self.flow_store.list_all_drafts(&realm_id).await
    }

    pub async fn publish_flow(&self, realm_id: Uuid, flow_id: Uuid) -> Result<FlowVersion> {
        self.publish_flow_with_tx(realm_id, flow_id, None).await
    }

    pub async fn publish_flow_with_tx(
        &self,
        realm_id: Uuid,
        flow_id: Uuid,
        mut tx: Option<&mut dyn Transaction>,
    ) -> Result<FlowVersion> {
        // 1. Get the Draft
        let draft = self.get_draft(flow_id).await?;

        // 2. Parse Draft JSON (String -> Value)
        let graph_json_value: serde_json::Value = serde_json::from_str(&draft.graph_json)
            .map_err(|e| Error::Validation(format!("Draft JSON is corrupted: {}", e)))?;

        // 3. Compile (Validates the graph logic)
        let execution_plan = FlowCompiler::compile(graph_json_value, &self.runtime_registry)?;

        // 4. Serialize Artifact (Struct -> String)
        let execution_artifact = serde_json::to_string(&execution_plan)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Serialization error: {}", e)))?;

        // 5. Calculate Next Version Number
        let current_max = self.flow_store.get_latest_version_number(&flow_id).await?;
        let next_version = current_max.map(|v| v + 1).unwrap_or(1); // Start at v1

        // 6. Ensure Parent Flow Exists (Promote Draft to Runtime if needed)
        // If this is a new custom flow, it might only exist in `flow_drafts`.
        // We need to ensure it exists in `auth_flows` before we attach a version to it.
        if self.flow_repo.find_flow_by_id(&flow_id).await?.is_none() {
            // Create the persistent flow record
            let new_flow = crate::domain::auth_flow::AuthFlow {
                id: flow_id,
                realm_id,
                name: draft.name.clone(),
                alias: draft.name.clone(),
                description: draft.description.clone(),
                r#type: draft.flow_type.clone(),
                built_in: false, // Custom flows are not built-in
            };
            // You need to expose create_flow in FlowStore if not already
            let tx_ref = tx.as_deref_mut();
            self.flow_repo.create_flow(&new_flow, tx_ref).await?;
        }

        // 7. Create Version Record
        let version = FlowVersion {
            id: Uuid::new_v4().to_string(),
            flow_id: flow_id.to_string(),
            version_number: next_version,
            execution_artifact,
            graph_json: draft.graph_json.clone(),
            checksum: "TODO_HASH".to_string(),
            created_at: Utc::now(),
        };
        let tx_ref = tx.as_deref_mut();
        self.flow_store
            .create_version_with_tx(&version, tx_ref)
            .await?;

        // 8. Update Deployment (Point LIVE to this version)
        let deployment = FlowDeployment {
            id: Uuid::new_v4().to_string(),
            realm_id,
            flow_type: draft.flow_type.clone(),
            active_version_id: version.id.clone(),
            updated_at: Utc::now(),
        };
        let tx_ref = tx.as_deref_mut();
        self.flow_store
            .set_deployment_with_tx(&deployment, tx_ref)
            .await?;

        let column_to_update = match draft.flow_type.as_str() {
            "browser" => Some("browser_flow_id"),
            "registration" => Some("registration_flow_id"),
            "direct" => Some("direct_grant_flow_id"),
            "reset" => Some("reset_credentials_flow_id"),
            "client" => Some("client_authentication_flow_id"),
            "docker" => Some("docker_authentication_flow_id"),
            _ => None, // Unknown or sub-flow types are not auto-bound
        };

        if let Some(col_name) = column_to_update {
            debug!("Auto-binding flow {} to realm slot {}", flow_id, col_name);

            // Pass references (&) for IDs and None for the transaction
            self.realm_repo
                .update_flow_binding(&realm_id, col_name, &flow_id, None)
                .await?;
        }

        // 9. Cleanup: Delete the draft
        // Now safe because flow_versions references auth_flows, not flow_drafts
        let tx_ref = tx.as_deref_mut();
        self.flow_store
            .delete_draft_with_tx(&flow_id, tx_ref)
            .await?;

        Ok(version)
    }

    pub async fn get_deployed_version(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
        flow_id: &Uuid,
    ) -> Result<Option<i32>> {
        self.flow_store
            .get_deployed_version_number(realm_id, flow_type, flow_id)
            .await
    }

    pub async fn is_flow_built_in(&self, flow_id: &Uuid) -> Result<bool> {
        let meta = self.flow_repo.find_flow_by_id(flow_id).await?;
        Ok(meta.map(|f| f.built_in).unwrap_or(false))
    }

    pub async fn list_flow_versions(
        &self,
        flow_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<FlowVersion>> {
        self.flow_store.list_versions(&flow_id, &req).await
    }

    pub fn generate_default_graph(flow_type: &str) -> String {
        let json_value = match flow_type {
            "browser" => FlowTemplates::browser_flow(),
            "direct" => FlowTemplates::direct_grant_flow(),
            "registration" => FlowTemplates::registration_flow(),
            "reset" => FlowTemplates::reset_credentials_flow(),
            // Fallback to a minimal graph
            _ => serde_json::json!({
                "nodes": [],
                "edges": []
            }),
        };

        // Convert to string for storage
        json_value.to_string()
    }

    pub async fn rollback_flow(
        &self,
        realm_id: Uuid,
        flow_id: Uuid,
        target_version: i32,
    ) -> Result<()> {
        // 1. Find the target version
        let version = self
            .flow_store
            .get_version_by_number(&flow_id, target_version)
            .await?
            .ok_or(Error::Unexpected(anyhow::anyhow!(
                "Version {} not found",
                target_version
            )))?;

        // 2. Get the flow metadata to know the type (browser, direct, etc.)
        let flow = self
            .flow_repo
            .find_flow_by_id(&flow_id)
            .await?
            .ok_or(Error::FlowNotFound(flow_id.to_string()))?;

        // 3. Update the Deployment to point to this old version
        // This is non-destructive; it just changes the pointer.
        let deployment = FlowDeployment {
            id: Uuid::new_v4().to_string(),
            realm_id,
            flow_type: flow.r#type,
            active_version_id: version.id, // Pointing to the OLD version ID
            updated_at: Utc::now(),
        };

        self.flow_store.set_deployment(&deployment).await?;

        // Note: We deliberately DO NOT overwrite the current draft.
        // A rollback is a runtime emergency action; it shouldn't destroy the user's work-in-progress.

        Ok(())
    }

    pub async fn restore_draft_from_version(
        &self,
        _realm_id: Uuid,
        flow_id: Uuid,
        version_number: i32,
    ) -> Result<()> {
        // 1. Fetch the target version
        let version = self
            .flow_store
            .get_version_by_number(&flow_id, version_number)
            .await?
            .ok_or(Error::Unexpected(anyhow::anyhow!(
                "Version {} not found",
                version_number
            )))?;

        // 2. Fetch the current draft (to preserve ID, name, etc.)
        let mut draft = self.get_draft(flow_id).await?;

        // 3. Overwrite ONLY the graph
        draft.graph_json = version.graph_json;
        draft.updated_at = Utc::now();

        // 4. Save
        self.flow_store.update_draft(&draft).await?;

        Ok(())
    }
}
