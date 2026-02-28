use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
use crate::ports::transaction_manager::Transaction;
use crate::{
    domain::pagination::{PageRequest, PageResponse},
    error::Result,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FlowStore: Send + Sync {
    // --- Drafts ---
    async fn create_draft(&self, draft: &FlowDraft) -> Result<()>;
    async fn create_draft_with_tx(
        &self,
        draft: &FlowDraft,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.create_draft(draft).await
    }
    async fn update_draft(&self, draft: &FlowDraft) -> Result<()>;
    async fn update_draft_with_tx(
        &self,
        draft: &FlowDraft,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.update_draft(draft).await
    }
    async fn get_draft_by_id(&self, id: &Uuid) -> Result<Option<FlowDraft>>;
    async fn get_draft_by_id_with_tx(
        &self,
        id: &Uuid,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<Option<FlowDraft>> {
        self.get_draft_by_id(id).await
    }
    async fn list_drafts(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowDraft>>;

    async fn list_all_drafts(&self, realm_id: &Uuid) -> Result<Vec<FlowDraft>>;
    async fn delete_draft(&self, id: &Uuid) -> Result<()>;
    async fn delete_draft_with_tx(
        &self,
        id: &Uuid,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.delete_draft(id).await
    }

    // --- Versions ---
    async fn create_version(&self, version: &FlowVersion) -> Result<()>;
    async fn create_version_with_tx(
        &self,
        version: &FlowVersion,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.create_version(version).await
    }
    async fn get_version(&self, id: &Uuid) -> Result<Option<FlowVersion>>;
    async fn list_versions(
        &self,
        flow_id: &Uuid,
        req: &PageRequest,
    ) -> Result<PageResponse<FlowVersion>>;

    // --- Deployments ---
    async fn set_deployment(&self, deployment: &FlowDeployment) -> Result<()>;
    async fn set_deployment_with_tx(
        &self,
        deployment: &FlowDeployment,
        _tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        self.set_deployment(deployment).await
    }
    async fn get_deployment(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
    ) -> Result<Option<FlowDeployment>>;
    async fn get_latest_version_number(&self, flow_id: &Uuid) -> Result<Option<i32>>;
    async fn get_latest_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>>;
    async fn get_deployed_version_number(
        &self,
        realm_id: &Uuid,
        flow_type: &str,
        flow_id: &Uuid,
    ) -> Result<Option<i32>>;

    async fn get_version_by_number(
        &self,
        flow_id: &Uuid,
        version_number: i32,
    ) -> Result<Option<FlowVersion>>;

    async fn get_active_version(&self, flow_id: &Uuid) -> Result<Option<FlowVersion>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::flow::models::{FlowDeployment, FlowDraft, FlowVersion};
    use crate::domain::pagination::{PageRequest, PageResponse};
    use crate::error::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    struct MockFlowStore {
        draft_created: Arc<Mutex<bool>>,
        draft_updated: Arc<Mutex<bool>>,
        draft_deleted: Arc<Mutex<bool>>,
        version_created: Arc<Mutex<bool>>,
        deployment_set: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl FlowStore for MockFlowStore {
        async fn create_draft(&self, _draft: &FlowDraft) -> Result<()> {
            let mut created = self.draft_created.lock().await;
            *created = true;
            Ok(())
        }
        async fn update_draft(&self, _draft: &FlowDraft) -> Result<()> {
            let mut updated = self.draft_updated.lock().await;
            *updated = true;
            Ok(())
        }
        async fn delete_draft(&self, _id: &Uuid) -> Result<()> {
            let mut deleted = self.draft_deleted.lock().await;
            *deleted = true;
            Ok(())
        }
        async fn create_version(&self, _version: &FlowVersion) -> Result<()> {
            let mut created = self.version_created.lock().await;
            *created = true;
            Ok(())
        }
        async fn set_deployment(&self, _deployment: &FlowDeployment) -> Result<()> {
            let mut set = self.deployment_set.lock().await;
            *set = true;
            Ok(())
        }

        // Unimplemented but needed for trait
        async fn get_draft_by_id(&self, _id: &Uuid) -> Result<Option<FlowDraft>> {
            unimplemented!()
        }
        async fn list_drafts(
            &self,
            _realm_id: &Uuid,
            _req: &PageRequest,
        ) -> Result<PageResponse<FlowDraft>> {
            unimplemented!()
        }
        async fn list_all_drafts(&self, _realm_id: &Uuid) -> Result<Vec<FlowDraft>> {
            unimplemented!()
        }
        async fn get_version(&self, _id: &Uuid) -> Result<Option<FlowVersion>> {
            unimplemented!()
        }
        async fn list_versions(
            &self,
            _flow_id: &Uuid,
            _req: &PageRequest,
        ) -> Result<PageResponse<FlowVersion>> {
            unimplemented!()
        }
        async fn get_deployment(
            &self,
            _realm_id: &Uuid,
            _flow_type: &str,
        ) -> Result<Option<FlowDeployment>> {
            unimplemented!()
        }
        async fn get_latest_version_number(&self, _flow_id: &Uuid) -> Result<Option<i32>> {
            unimplemented!()
        }
        async fn get_latest_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
            unimplemented!()
        }
        async fn get_deployed_version_number(
            &self,
            _realm_id: &Uuid,
            _flow_type: &str,
            _flow_id: &Uuid,
        ) -> Result<Option<i32>> {
            unimplemented!()
        }
        async fn get_version_by_number(
            &self,
            _flow_id: &Uuid,
            _version_number: i32,
        ) -> Result<Option<FlowVersion>> {
            unimplemented!()
        }
        async fn get_active_version(&self, _flow_id: &Uuid) -> Result<Option<FlowVersion>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_flow_store_default_methods() {
        let mock = MockFlowStore {
            draft_created: Arc::new(Mutex::new(false)),
            draft_updated: Arc::new(Mutex::new(false)),
            draft_deleted: Arc::new(Mutex::new(false)),
            version_created: Arc::new(Mutex::new(false)),
            deployment_set: Arc::new(Mutex::new(false)),
        };

        let realm_id = Uuid::new_v4();
        let flow_id = Uuid::new_v4();
        let now = Utc::now();
        let draft = FlowDraft {
            id: Uuid::new_v4(),
            realm_id,
            name: "test".to_string(),
            description: None,
            graph_json: "[]".to_string(),
            flow_type: "browser".to_string(),
            created_at: now,
            updated_at: now,
        };
        let version = FlowVersion {
            id: "v1".to_string(),
            flow_id: flow_id.to_string(),
            version_number: 1,
            execution_artifact: "[]".to_string(),
            graph_json: "[]".to_string(),
            checksum: "sum".to_string(),
            created_at: now,
        };
        let deployment = FlowDeployment {
            id: "d1".to_string(),
            realm_id,
            flow_type: "browser".to_string(),
            active_version_id: "v1".to_string(),
            updated_at: now,
        };

        mock.create_draft_with_tx(&draft, None)
            .await
            .expect("create draft");
        assert!(*mock.draft_created.lock().await);

        mock.update_draft_with_tx(&draft, None)
            .await
            .expect("update draft");
        assert!(*mock.draft_updated.lock().await);

        mock.delete_draft_with_tx(&draft.id, None)
            .await
            .expect("delete draft");
        assert!(*mock.draft_deleted.lock().await);

        mock.create_version_with_tx(&version, None)
            .await
            .expect("create version");
        assert!(*mock.version_created.lock().await);

        mock.set_deployment_with_tx(&deployment, None)
            .await
            .expect("set deployment");
        assert!(*mock.deployment_set.lock().await);
    }
}
