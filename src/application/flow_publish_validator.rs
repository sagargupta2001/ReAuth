use crate::application::node_registry::NodeRegistryService;
use crate::application::theme_service::ThemeResolverService;
use crate::domain::flow::models::NodeMetadata;
use crate::domain::ui::PageCategory;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait FlowPublishValidator: Send + Sync {
    async fn validate(&self, realm_id: Uuid, graph: &Value) -> Result<()>;
}

pub struct UiBindingPublishValidator {
    theme_service: Arc<ThemeResolverService>,
    node_registry: Arc<NodeRegistryService>,
}

impl UiBindingPublishValidator {
    pub fn new(
        theme_service: Arc<ThemeResolverService>,
        node_registry: Arc<NodeRegistryService>,
    ) -> Self {
        Self {
            theme_service,
            node_registry,
        }
    }
}

#[async_trait]
impl FlowPublishValidator for UiBindingPublishValidator {
    async fn validate(&self, realm_id: Uuid, graph: &Value) -> Result<()> {
        let nodes = graph
            .get("nodes")
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::Validation("Missing nodes".to_string()))?;

        let binding = self.theme_service.resolve_binding(realm_id, None).await?;
        let pages = if let Some(binding) = binding {
            self.theme_service
                .list_pages_for_theme(realm_id, binding.theme_id)
                .await?
        } else {
            self.theme_service.list_pages()
        };

        let pages_by_key: HashMap<String, PageCategory> = pages
            .into_iter()
            .map(|page| (page.key, page.category))
            .collect();

        let metadata_by_id: HashMap<String, NodeMetadata> = self
            .node_registry
            .get_available_nodes()
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();

        let mut missing_pages: HashMap<String, Vec<String>> = HashMap::new();
        let mut missing_bindings: Vec<String> = Vec::new();
        let mut category_mismatches: Vec<String> = Vec::new();

        for node in nodes {
            let node_type = node
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let metadata = match metadata_by_id.get(node_type) {
                Some(meta) if meta.supports_ui => meta,
                _ => continue,
            };

            let node_id = node
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");

            let config = node.get("data").and_then(|value| value.get("config"));
            let explicit = config
                .and_then(|value| value.get("ui"))
                .and_then(|value| value.get("page_key"))
                .and_then(|value| value.as_str())
                .or_else(|| {
                    config
                        .and_then(|value| value.get("template_key"))
                        .and_then(|value| value.as_str())
                })
                .map(str::trim)
                .filter(|value| !value.is_empty());

            let fallback = metadata.default_template_key.as_deref();
            let Some(page_key) = explicit.or(fallback) else {
                missing_bindings.push(format!("node_id={} ({})", node_id, node_type));
                continue;
            };

            let Some(category) = pages_by_key.get(page_key) else {
                missing_pages
                    .entry(page_key.to_string())
                    .or_default()
                    .push(node_id.to_string());
                continue;
            };

            if category == &PageCategory::Custom {
                continue;
            }

            if !metadata.allowed_page_categories.is_empty()
                && !metadata.allowed_page_categories.contains(category)
            {
                category_mismatches.push(format!(
                    "node_id={} ({}) uses '{}' ({}) but allows {}",
                    node_id,
                    node_type,
                    page_key,
                    page_category_label(category),
                    format_categories(&metadata.allowed_page_categories),
                ));
            }
        }

        if missing_pages.is_empty() && missing_bindings.is_empty() && category_mismatches.is_empty()
        {
            return Ok(());
        }

        let mut parts = Vec::new();
        if !missing_pages.is_empty() {
            let mut list: Vec<String> = missing_pages
                .into_iter()
                .map(|(key, mut nodes)| {
                    nodes.sort();
                    nodes.dedup();
                    if nodes.is_empty() {
                        key
                    } else {
                        let tagged = nodes
                            .into_iter()
                            .map(|node| format!("node_id={}", node))
                            .collect::<Vec<String>>()
                            .join(", ");
                        format!("{} (nodes: {})", key, tagged)
                    }
                })
                .collect();
            list.sort();
            parts.push(format!("Missing theme pages: {}", list.join(", ")));
        }
        if !missing_bindings.is_empty() {
            parts.push(format!(
                "UI nodes missing page binding: {}",
                missing_bindings.join(", ")
            ));
        }
        if !category_mismatches.is_empty() {
            parts.push(format!(
                "Page category mismatches: {}",
                category_mismatches.join(" | ")
            ));
        }

        Err(Error::Validation(parts.join(" | ")))
    }
}

fn page_category_label(category: &PageCategory) -> &'static str {
    match category {
        PageCategory::Auth => "auth",
        PageCategory::Consent => "consent",
        PageCategory::AwaitingAction => "awaiting_action",
        PageCategory::Verification => "verification",
        PageCategory::Mfa => "mfa",
        PageCategory::Notification => "notification",
        PageCategory::Error => "error",
        PageCategory::Custom => "custom",
    }
}

fn format_categories(categories: &[PageCategory]) -> String {
    let mut labels: Vec<&'static str> = categories.iter().map(page_category_label).collect();
    labels.sort();
    labels.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::node_registry::NodeRegistryService;
    use crate::application::runtime_registry::RuntimeRegistry;
    use crate::application::theme_service::ThemeResolverService;
    use crate::domain::execution::StepType;
    use crate::domain::flow::nodes::oidc_consent_node::OidcConsentNodeProvider;
    use crate::domain::flow::nodes::password_node::PasswordNodeProvider;
    use crate::domain::flow::provider::NodeProvider;
    use crate::domain::theme::{
        Theme, ThemeAsset, ThemeAssetMeta, ThemeBinding, ThemeLayout, ThemeNode, ThemeTokens,
        ThemeVersion,
    };
    use crate::ports::theme_repository::ThemeRepository;
    use crate::ports::transaction_manager::{Transaction, TransactionManager};
    use async_trait::async_trait;
    use serde_json::json;
    use std::any::Any;
    use std::sync::Mutex;

    struct NoDefaultUiNodeProvider;

    impl NodeProvider for NoDefaultUiNodeProvider {
        fn id(&self) -> &'static str {
            "core.ui.no_default"
        }

        fn display_name(&self) -> &'static str {
            "No Default UI"
        }

        fn description(&self) -> &'static str {
            "UI node without a default page binding."
        }

        fn icon(&self) -> &'static str {
            "AlertCircle"
        }

        fn category(&self) -> &'static str {
            "Authenticator"
        }

        fn outputs(&self) -> Vec<&'static str> {
            vec!["success"]
        }

        fn config_schema(&self) -> Value {
            json!({})
        }

        fn supports_ui(&self) -> bool {
            true
        }

        fn allowed_page_categories(&self) -> Vec<PageCategory> {
            vec![PageCategory::Auth]
        }
    }

    #[derive(Clone)]
    struct TestThemeRepo {
        theme: Theme,
        binding: Option<ThemeBinding>,
        nodes: Vec<ThemeNode>,
    }

    impl TestThemeRepo {
        fn new(theme: Theme, binding: Option<ThemeBinding>, nodes: Vec<ThemeNode>) -> Self {
            Self {
                theme,
                binding,
                nodes,
            }
        }
    }

    #[async_trait]
    impl ThemeRepository for TestThemeRepo {
        async fn create_theme(
            &self,
            _theme: &Theme,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_theme(
            &self,
            _theme: &Theme,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn set_theme_system(
            &self,
            _theme_id: &Uuid,
            _is_system: bool,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn delete_theme(
            &self,
            _realm_id: &Uuid,
            _theme_id: &Uuid,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn find_theme(&self, _realm_id: &Uuid, theme_id: &Uuid) -> Result<Option<Theme>> {
            Ok((theme_id == &self.theme.id).then(|| self.theme.clone()))
        }

        async fn list_themes(&self, _realm_id: &Uuid) -> Result<Vec<Theme>> {
            Ok(vec![self.theme.clone()])
        }

        async fn upsert_tokens(
            &self,
            _tokens: &ThemeTokens,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_tokens(&self, _theme_id: &Uuid) -> Result<Option<ThemeTokens>> {
            Ok(None)
        }

        async fn upsert_layout(
            &self,
            _layout: &ThemeLayout,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_layout(&self, _theme_id: &Uuid, _name: &str) -> Result<Option<ThemeLayout>> {
            Ok(None)
        }

        async fn list_layouts(&self, _theme_id: &Uuid) -> Result<Vec<ThemeLayout>> {
            Ok(Vec::new())
        }

        async fn upsert_node(
            &self,
            _node: &ThemeNode,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_node(&self, _theme_id: &Uuid, _node_key: &str) -> Result<Option<ThemeNode>> {
            Ok(None)
        }

        async fn list_nodes(&self, _theme_id: &Uuid) -> Result<Vec<ThemeNode>> {
            Ok(self.nodes.clone())
        }

        async fn delete_node(
            &self,
            _theme_id: &Uuid,
            _node_key: &str,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn create_asset(
            &self,
            _asset: &ThemeAsset,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_asset(
            &self,
            _theme_id: &Uuid,
            _asset_id: &Uuid,
        ) -> Result<Option<ThemeAsset>> {
            Ok(None)
        }

        async fn list_assets(&self, _theme_id: &Uuid) -> Result<Vec<ThemeAssetMeta>> {
            Ok(Vec::new())
        }

        async fn delete_asset(
            &self,
            _theme_id: &Uuid,
            _asset_id: &Uuid,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn set_draft_exists(
            &self,
            _theme_id: &Uuid,
            _exists: bool,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_draft_exists(&self, _theme_id: &Uuid) -> Result<bool> {
            Ok(false)
        }

        async fn create_version(
            &self,
            _version: &ThemeVersion,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_version(
            &self,
            _theme_id: &Uuid,
            _version_id: &Uuid,
        ) -> Result<Option<ThemeVersion>> {
            Ok(None)
        }

        async fn list_versions(&self, _theme_id: &Uuid) -> Result<Vec<ThemeVersion>> {
            Ok(Vec::new())
        }

        async fn set_version_status(
            &self,
            _version_id: &Uuid,
            _status: &str,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn upsert_binding(
            &self,
            _binding: &ThemeBinding,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_binding(
            &self,
            realm_id: &Uuid,
            _client_id: Option<&str>,
        ) -> Result<Option<ThemeBinding>> {
            Ok(self
                .binding
                .clone()
                .filter(|binding| &binding.realm_id == realm_id))
        }

        async fn list_bindings(&self, _realm_id: &Uuid) -> Result<Vec<ThemeBinding>> {
            Ok(self.binding.clone().into_iter().collect())
        }

        async fn delete_binding(
            &self,
            _realm_id: &Uuid,
            _client_id: Option<&str>,
            _tx: Option<&mut dyn Transaction>,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct TestTx;

    impl Transaction for TestTx {
        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    #[derive(Default)]
    struct TestTxManager {
        begin_calls: Mutex<usize>,
        commit_calls: Mutex<usize>,
        rollback_calls: Mutex<usize>,
    }

    #[async_trait]
    impl TransactionManager for TestTxManager {
        async fn begin(&self) -> Result<Box<dyn Transaction>> {
            *self.begin_calls.lock().unwrap() += 1;
            Ok(Box::new(TestTx))
        }

        async fn commit(&self, _tx: Box<dyn Transaction>) -> Result<()> {
            *self.commit_calls.lock().unwrap() += 1;
            Ok(())
        }

        async fn rollback(&self, _tx: Box<dyn Transaction>) -> Result<()> {
            *self.rollback_calls.lock().unwrap() += 1;
            Ok(())
        }
    }

    fn build_validator() -> (UiBindingPublishValidator, Uuid) {
        let realm_id = Uuid::new_v4();
        let theme_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();
        let theme = Theme {
            id: theme_id,
            realm_id,
            name: "Test Theme".to_string(),
            description: None,
            is_system: true,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        };
        let binding = ThemeBinding {
            id: Uuid::new_v4(),
            realm_id,
            client_id: None,
            theme_id,
            active_version_id: version_id,
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        };
        let theme_repo = Arc::new(TestThemeRepo::new(theme, Some(binding), Vec::new()));
        let tx_manager = Arc::new(TestTxManager::default());
        let theme_service = Arc::new(ThemeResolverService::new(theme_repo, tx_manager));

        let mut registry = RuntimeRegistry::new();
        registry.register_definition("core.auth.password", StepType::Authenticator);
        registry.register_definition("core.oidc.consent", StepType::Authenticator);
        registry.register_definition("core.ui.no_default", StepType::Authenticator);
        let runtime_registry = Arc::new(registry);

        let providers: Vec<Box<dyn NodeProvider>> = vec![
            Box::new(PasswordNodeProvider),
            Box::new(OidcConsentNodeProvider),
            Box::new(NoDefaultUiNodeProvider),
        ];
        let node_registry = Arc::new(NodeRegistryService::with_providers(
            providers,
            runtime_registry,
        ));

        (
            UiBindingPublishValidator::new(theme_service, node_registry),
            realm_id,
        )
    }

    fn graph_with_node(node_type: &str, config: Value) -> Value {
        json!({
            "nodes": [
                {
                    "id": "node-1",
                    "type": node_type,
                    "data": { "config": config }
                }
            ],
            "edges": []
        })
    }

    #[tokio::test]
    async fn publish_validator_rejects_missing_page_key() {
        let (validator, realm_id) = build_validator();
        let graph = graph_with_node(
            "core.auth.password",
            json!({ "ui": { "page_key": "custom.missing" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        assert!(err.to_string().contains("Missing theme pages"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_category_mismatch() {
        let (validator, realm_id) = build_validator();
        let graph = graph_with_node(
            "core.oidc.consent",
            json!({ "ui": { "page_key": "login" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        assert!(err.to_string().contains("Page category mismatches"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_missing_binding() {
        let (validator, realm_id) = build_validator();
        let graph = graph_with_node("core.ui.no_default", json!({}));

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        assert!(err.to_string().contains("UI nodes missing page binding"));
    }
}
