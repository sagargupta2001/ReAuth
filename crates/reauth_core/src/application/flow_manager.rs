use crate::application::flow_manager;
use crate::domain::compiler::compiler::FlowCompiler;
use crate::domain::flow::{FlowDeployment, FlowVersion};
use crate::ports::flow_repository::FlowRepository;
use crate::{
    domain::{
        flow::FlowDraft,
        pagination::{PageRequest, PageResponse},
    },
    error::{Error, Result},
    ports::{flow_store::FlowStore, realm_repository::RealmRepository},
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
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
}

impl FlowManager {
    pub fn new(
        flow_store: Arc<dyn FlowStore>,
        flow_repo: Arc<dyn FlowRepository>,
        realm_repo: Arc<dyn RealmRepository>,
    ) -> Self {
        Self {
            flow_store,
            flow_repo,
            realm_repo,
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
        // 1. Get the Draft
        let draft = self.get_draft(flow_id).await?;

        // 2. Parse Draft JSON (String -> Value)
        let graph_json_value: serde_json::Value = serde_json::from_str(&draft.graph_json)
            .map_err(|e| Error::Validation(format!("Draft JSON is corrupted: {}", e)))?;

        // 3. Compile (Validates the graph logic)
        let execution_plan = FlowCompiler::compile(graph_json_value)?;

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
            self.flow_repo.create_flow(&new_flow, None).await?;
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
        self.flow_store.create_version(&version).await?;

        // 8. Update Deployment (Point LIVE to this version)
        let deployment = FlowDeployment {
            id: Uuid::new_v4().to_string(),
            realm_id,
            flow_type: draft.flow_type.clone(),
            active_version_id: version.id.clone(),
            updated_at: Utc::now(),
        };
        self.flow_store.set_deployment(&deployment).await?;

        // 9. Cleanup: Delete the draft
        // Now safe because flow_versions references auth_flows, not flow_drafts
        self.flow_store.delete_draft(&flow_id).await?;

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

    pub fn generate_default_graph(flow_type: &str) -> String {
        match flow_type {
            "browser" => r#"{
                "nodes": [
                    { "id": "start", "type": "default", "position": { "x": 250, "y": 0 }, "data": { "label": "Start", "config": {} } },
                    { "id": "auth-1", "type": "authenticator", "position": { "x": 250, "y": 150 }, "data": { "label": "Username Password", "config": {} } },
                    { "id": "end", "type": "terminal", "position": { "x": 250, "y": 300 }, "data": { "label": "Success", "config": {} } }
                ],
                "edges": [
                    { "id": "e1", "source": "start", "target": "auth-1" },
                    { "id": "e2", "source": "auth-1", "target": "end" }
                ]
            }"#.to_string(),

            "direct" => r#"{
                "nodes": [
                    { "id": "auth-1", "type": "authenticator", "position": { "x": 250, "y": 50 }, "data": { "label": "Direct Grant Auth", "config": {} } },
                    { "id": "end", "type": "terminal", "position": { "x": 250, "y": 200 }, "data": { "label": "Success", "config": {} } }
                ],
                "edges": [
                    { "id": "e1", "source": "auth-1", "target": "end" }
                ]
            }"#.to_string(),

            // Default empty structure
            _ => r#"{ "nodes": [], "edges": [] }"#.to_string(),
        }
    }
}
