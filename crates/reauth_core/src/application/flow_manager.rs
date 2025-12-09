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
}

#[derive(Deserialize)]
pub struct UpdateDraftRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub graph_json: Option<serde_json::Value>,
}

pub struct FlowManager {
    flow_store: Arc<dyn FlowStore>,
    realm_repo: Arc<dyn RealmRepository>,
}

impl FlowManager {
    pub fn new(flow_store: Arc<dyn FlowStore>, realm_repo: Arc<dyn RealmRepository>) -> Self {
        Self {
            flow_store,
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
            graph_json: "{}".to_string(), // Empty graph initially
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.flow_store.create_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn get_draft(&self, id: Uuid) -> Result<FlowDraft> {
        self.flow_store
            .get_draft_by_id(&id)
            .await?
            .ok_or(Error::Unexpected(anyhow::anyhow!("Flow draft not found"))) // Add specific error later
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
}
