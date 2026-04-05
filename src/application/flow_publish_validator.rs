use crate::application::node_registry::NodeRegistryService;
use crate::application::script_engine::{
    BoaScriptEngine, ScriptEngine, ScriptExecutionContext, ScriptExecutionLimits,
};
use crate::application::theme_service::ThemeResolverService;
use crate::domain::flow::models::{FlowPublishIssue, FlowPublishValidation, NodeContract};
use crate::domain::flow::signal::FlowSignal;
use crate::domain::theme_pages::ThemePageTemplate;
use crate::domain::ui::PageCategory;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
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

        let pages_by_key: HashMap<String, ThemePageTemplate> = pages
            .into_iter()
            .map(|page| (page.key.clone(), page))
            .collect();

        let metadata_by_id: HashMap<String, NodeContract> = self
            .node_registry
            .get_available_nodes()
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();

        let graph_node_ids: HashSet<String> = nodes
            .iter()
            .filter_map(|node| node.get("id").and_then(|value| value.as_str()))
            .map(|value| value.to_string())
            .collect();

        let mut used_pages: HashMap<String, Vec<String>> = HashMap::new();
        let mut missing_pages: HashMap<String, Vec<String>> = HashMap::new();
        let mut missing_bindings: Vec<(String, String)> = Vec::new();
        let mut category_mismatches: Vec<(String, String)> = Vec::new();
        let mut signal_type_errors: Vec<(String, Vec<String>)> = Vec::new();
        let mut signal_node_errors: Vec<(String, Vec<String>)> = Vec::new();
        let mut signal_parse_errors: Vec<(String, Vec<String>)> = Vec::new();
        let mut payload_map_errors: Vec<(String, Vec<String>)> = Vec::new();
        let mut scripted_ui_errors: Vec<(String, Vec<String>)> = Vec::new();

        let script_engine = BoaScriptEngine;

        for node in nodes {
            let node_type = node
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let metadata = match metadata_by_id.get(node_type) {
                Some(meta) if meta.capabilities.supports_ui => meta,
                _ => continue,
            };

            let node_id = node
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");

            let config = node.get("data").and_then(|value| value.get("config"));

            if node_type == "core.ui.scripted" {
                if let Some(message) =
                    dry_run_scripted_ui(&script_engine, config, node_id, &pages_by_key)
                {
                    scripted_ui_errors.push((message, vec![node_id.to_string()]));
                }
            }
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
                missing_bindings.push((node_id.to_string(), node_type.to_string()));
                continue;
            };

            used_pages
                .entry(page_key.to_string())
                .or_default()
                .push(node_id.to_string());

            let Some(template) = pages_by_key.get(page_key) else {
                missing_pages
                    .entry(page_key.to_string())
                    .or_default()
                    .push(node_id.to_string());
                continue;
            };

            let category = &template.category;
            if category == &PageCategory::Custom {
                continue;
            }

            if !metadata.capabilities.allowed_page_categories.is_empty()
                && !metadata
                    .capabilities
                    .allowed_page_categories
                    .contains(category)
            {
                category_mismatches.push((
                    node_id.to_string(),
                    format!(
                        "node_id={} ({}) uses '{}' ({}) but allows {}",
                        node_id,
                        node_type,
                        page_key,
                        page_category_label(category),
                        format_categories(&metadata.capabilities.allowed_page_categories),
                    ),
                ));
            }
        }

        for (page_key, nodes_for_page) in &used_pages {
            let Some(template) = pages_by_key.get(page_key) else {
                continue;
            };
            let mut signal_values = Vec::new();
            collect_signal_bindings(&template.blueprint, &mut signal_values);
            let mut input_names = HashSet::new();
            collect_input_names(&template.blueprint, &mut input_names);
            if signal_values.is_empty() {
                continue;
            }

            let node_tags = format_node_tags(nodes_for_page);
            for signal_value in signal_values {
                let signal: FlowSignal = match serde_json::from_value(signal_value.clone()) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        signal_parse_errors.push((
                            format!(
                                "page '{}' (nodes: {}) invalid signal ({})",
                                page_key, node_tags, err
                            ),
                            nodes_for_page.clone(),
                        ));
                        continue;
                    }
                };

                if let Some(message) = validate_payload_map(&signal_value, &input_names) {
                    payload_map_errors.push((
                        format!("page '{}' (nodes: {}) {}", page_key, node_tags, message),
                        nodes_for_page.clone(),
                    ));
                }

                if !signal.is_allowed_type() {
                    signal_type_errors.push((
                        format!(
                            "page '{}' (nodes: {}) uses '{}'",
                            page_key, node_tags, signal.signal_type
                        ),
                        nodes_for_page.clone(),
                    ));
                }

                if let Some(node_id) = signal.normalized_node_id() {
                    if !graph_node_ids.contains(node_id) {
                        signal_node_errors.push((
                            format!(
                                "page '{}' (nodes: {}) references node_id={}",
                                page_key, node_tags, node_id
                            ),
                            nodes_for_page.clone(),
                        ));
                    }
                }
            }
        }

        if missing_pages.is_empty()
            && missing_bindings.is_empty()
            && category_mismatches.is_empty()
            && signal_type_errors.is_empty()
            && signal_node_errors.is_empty()
            && signal_parse_errors.is_empty()
            && payload_map_errors.is_empty()
            && scripted_ui_errors.is_empty()
        {
            return Ok(());
        }

        let mut issues: Vec<FlowPublishIssue> = Vec::new();
        let mut parts = Vec::new();
        if !missing_pages.is_empty() {
            let mut list: Vec<String> = missing_pages
                .into_iter()
                .map(|(key, mut nodes)| {
                    nodes.sort();
                    nodes.dedup();
                    issues.push(FlowPublishIssue {
                        message: format!("Missing theme page '{}'", key),
                        node_ids: nodes.clone(),
                    });
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
            for (node_id, node_type) in &missing_bindings {
                issues.push(FlowPublishIssue {
                    message: format!(
                        "UI node missing page binding: node_id={} ({})",
                        node_id, node_type
                    ),
                    node_ids: vec![node_id.clone()],
                });
            }
            parts.push(format!(
                "UI nodes missing page binding: {}",
                missing_bindings
                    .iter()
                    .map(|(node_id, node_type)| format!("node_id={} ({})", node_id, node_type))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }
        if !category_mismatches.is_empty() {
            for (node_id, message) in &category_mismatches {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: vec![node_id.clone()],
                });
            }
            parts.push(format!(
                "Page category mismatches: {}",
                category_mismatches
                    .iter()
                    .map(|(_, message)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }
        if !signal_type_errors.is_empty() {
            for (message, node_ids) in &signal_type_errors {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: node_ids.clone(),
                });
            }
            parts.push(format!(
                "Signal bindings use unsupported types: {}",
                signal_type_errors
                    .iter()
                    .map(|(message, _)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }
        if !signal_node_errors.is_empty() {
            for (message, node_ids) in &signal_node_errors {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: node_ids.clone(),
                });
            }
            parts.push(format!(
                "Signal bindings reference missing nodes: {}",
                signal_node_errors
                    .iter()
                    .map(|(message, _)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }
        if !signal_parse_errors.is_empty() {
            for (message, node_ids) in &signal_parse_errors {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: node_ids.clone(),
                });
            }
            parts.push(format!(
                "Signal bindings invalid: {}",
                signal_parse_errors
                    .iter()
                    .map(|(message, _)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }
        if !payload_map_errors.is_empty() {
            for (message, node_ids) in &payload_map_errors {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: node_ids.clone(),
                });
            }
            parts.push(format!(
                "Signal payload_map invalid: {}",
                payload_map_errors
                    .iter()
                    .map(|(message, _)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }
        if !scripted_ui_errors.is_empty() {
            for (message, node_ids) in &scripted_ui_errors {
                issues.push(FlowPublishIssue {
                    message: message.clone(),
                    node_ids: node_ids.clone(),
                });
            }
            parts.push(format!(
                "Scripted UI validation failed: {}",
                scripted_ui_errors
                    .iter()
                    .map(|(message, _)| message.clone())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ));
        }

        Err(Error::FlowPublishValidation(FlowPublishValidation {
            message: parts.join(" | "),
            issues,
        }))
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

fn collect_signal_bindings(value: &Value, signals: &mut Vec<Value>) {
    match value {
        Value::Array(values) => {
            for item in values {
                collect_signal_bindings(item, signals);
            }
        }
        Value::Object(map) => {
            if let Some(signal) = map.get("signal") {
                signals.push(signal.clone());
            }
            for value in map.values() {
                collect_signal_bindings(value, signals);
            }
        }
        _ => {}
    }
}

fn collect_input_names(value: &Value, names: &mut HashSet<String>) {
    match value {
        Value::Array(values) => {
            for item in values {
                collect_input_names(item, names);
            }
        }
        Value::Object(map) => {
            if let Some(Value::String(node_type)) = map.get("type") {
                let is_input = if node_type == "Input" {
                    true
                } else if node_type == "Component" {
                    map.get("component")
                        .and_then(Value::as_str)
                        .is_some_and(|value| value.eq_ignore_ascii_case("input"))
                } else {
                    false
                };

                if is_input {
                    if let Some(Value::Object(props)) = map.get("props") {
                        if let Some(Value::String(name)) = props.get("name") {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                names.insert(trimmed.to_string());
                            }
                        }
                    }
                }
            }
            for value in map.values() {
                collect_input_names(value, names);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Deserialize)]
struct ScriptedUiConfig {
    script: Option<String>,
    #[allow(dead_code)]
    screen_id: Option<String>,
    ui_context: Option<Value>,
    signal_handlers: Option<HashMap<String, String>>,
}

fn dry_run_scripted_ui(
    engine: &dyn ScriptEngine,
    config: Option<&Value>,
    node_id: &str,
    pages_by_key: &HashMap<String, ThemePageTemplate>,
) -> Option<String> {
    let config_value = config
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    let parsed: ScriptedUiConfig = match serde_json::from_value(config_value) {
        Ok(value) => value,
        Err(err) => {
            return Some(format!(
                "node_id={} scripted_ui config invalid ({})",
                node_id, err
            ))
        }
    };

    let mut scripts: Vec<(String, String, Option<String>)> = Vec::new();
    if let Some(script) = parsed
        .script
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        scripts.push(("default".to_string(), script, None));
    }
    if let Some(handlers) = parsed.signal_handlers {
        for (signal_type, script) in handlers {
            if script.trim().is_empty() {
                continue;
            }
            scripts.push((format!("signal:{}", signal_type), script, Some(signal_type)));
        }
    }

    for (label, script, signal_type) in scripts {
        let signal = if let Some(signal_type) = signal_type {
            serde_json::json!({ "type": signal_type })
        } else {
            serde_json::json!({})
        };
        let ctx = ScriptExecutionContext {
            input: serde_json::json!({}),
            context: parsed
                .ui_context
                .clone()
                .unwrap_or_else(|| serde_json::json!({})),
            signal,
        };
        let result = match engine.execute(&script, ctx, ScriptExecutionLimits::for_ui()) {
            Ok(value) => value,
            Err(err) => {
                return Some(format!(
                    "node_id={} scripted_ui {} failed ({})",
                    node_id, label, err
                ))
            }
        };
        if let Some(message) = validate_scripted_ui_result(&result, pages_by_key) {
            return Some(format!(
                "node_id={} scripted_ui {} invalid ({})",
                node_id, label, message
            ));
        }
    }

    None
}

fn validate_scripted_ui_result(
    value: &Value,
    pages_by_key: &HashMap<String, ThemePageTemplate>,
) -> Option<String> {
    let Value::Object(map) = value else {
        return Some("scripted UI handler must return an object".to_string());
    };

    if let Some(Value::Object(context)) = map.get("context") {
        if let Some(Value::String(template_key)) = context.get("template_key") {
            let trimmed = template_key.trim();
            if !trimmed.is_empty() && !pages_by_key.contains_key(trimmed) {
                return Some(format!("unknown template_key '{}'", trimmed));
            }
        } else if let Some(value) = context.get("template_key") {
            if !value.is_null() {
                return Some("template_key must be a string".to_string());
            }
        }

        if let Some(patch) = context
            .get("ui_patch")
            .or_else(|| context.get("fluid_patch"))
            .or_else(|| context.get("patch"))
        {
            if let Err(err) = validate_fluid_patch(patch) {
                return Some(err.to_string());
            }
        }
    } else if let Some(value) = map.get("context") {
        if !value.is_null() {
            return Some("context must be an object".to_string());
        }
    }

    None
}

fn validate_fluid_patch(value: &Value) -> Result<()> {
    match value {
        Value::Array(nodes) => {
            for node in nodes {
                validate_node_value(node)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            if let Some(nodes) = map.get("nodes") {
                if let Value::Array(node_list) = nodes {
                    for node in node_list {
                        validate_node_value(node)?;
                    }
                    return Ok(());
                }
                return Err(Error::Validation(
                    "UI patch nodes must be an array".to_string(),
                ));
            }
            Err(Error::Validation(
                "UI patch must contain a nodes array".to_string(),
            ))
        }
        _ => Err(Error::Validation(
            "UI patch must be an array or object".to_string(),
        )),
    }
}

fn validate_node_value(value: &Value) -> Result<()> {
    let obj = value
        .as_object()
        .ok_or_else(|| Error::Validation("Theme node must be an object".to_string()))?;
    let node_type = obj
        .get("type")
        .and_then(|value| value.as_str())
        .ok_or_else(|| Error::Validation("Theme node type is required".to_string()))?;

    let allowed = ["Box", "Text", "Image", "Icon", "Input", "Component"];
    if !allowed.contains(&node_type) {
        return Err(Error::Validation(format!(
            "Unsupported node type: {}",
            node_type
        )));
    }

    if node_type == "Component" {
        let component = obj
            .get("component")
            .and_then(|value| value.as_str())
            .ok_or_else(|| Error::Validation("Component type is required".to_string()))?;

        let allowed_components = ["Button", "Divider", "Input", "Link", "SocialProvider"];
        if !allowed_components.contains(&component) {
            return Err(Error::Validation(format!(
                "Unsupported component type: {}",
                component
            )));
        }
    }

    if let Some(children) = obj.get("children") {
        if let Value::Array(nodes) = children {
            for node in nodes {
                validate_node_value(node)?;
            }
        } else {
            return Err(Error::Validation(
                "Theme node children must be an array".to_string(),
            ));
        }
    }

    if let Some(slots) = obj.get("slots") {
        if let Value::Object(slot_map) = slots {
            for node in slot_map.values() {
                validate_node_value(node)?;
            }
        } else {
            return Err(Error::Validation(
                "Theme node slots must be an object".to_string(),
            ));
        }
    }

    Ok(())
}

fn format_node_tags(nodes: &[String]) -> String {
    if nodes.is_empty() {
        return "none".to_string();
    }
    let mut list = nodes.to_vec();
    list.sort();
    list.dedup();
    list.into_iter()
        .map(|node| format!("node_id={}", node))
        .collect::<Vec<String>>()
        .join(", ")
}

fn validate_payload_map(signal_value: &Value, input_names: &HashSet<String>) -> Option<String> {
    let payload_map = signal_value.get("payload_map")?;

    match payload_map {
        Value::Null => None,
        Value::Object(map) => {
            for (key, value) in map {
                let trimmed_key = key.trim();
                if trimmed_key.is_empty() {
                    return Some("payload_map contains an empty key".to_string());
                }
                let Value::String(path) = value else {
                    return Some(format!(
                        "payload_map entry '{}' must be a string path",
                        trimmed_key
                    ));
                };
                if !is_valid_payload_path(path, input_names) {
                    return Some(format!(
                        "payload_map entry '{}' has invalid path '{}'",
                        trimmed_key, path
                    ));
                }
            }
            None
        }
        _ => Some("payload_map must be an object".to_string()),
    }
}

fn is_valid_payload_path(path: &str, input_names: &HashSet<String>) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    if let Some(rest) = trimmed.strip_prefix("inputs.") {
        let mut parts = rest.split('.');
        let name = parts.next().unwrap_or_default().trim();
        if name.is_empty() {
            return false;
        }
        if !input_names.contains(name) {
            return false;
        }
        return parts.all(|part| !part.trim().is_empty());
    }
    if trimmed == "inputs" {
        return false;
    }
    if let Some(rest) = trimmed.strip_prefix("context.") {
        return !rest.trim().is_empty();
    }
    false
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
    use crate::domain::flow::nodes::scripted_ui_node::ScriptedUiNodeProvider;
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
        build_validator_with_nodes(Vec::new())
    }

    fn build_validator_with_nodes(nodes: Vec<ThemeNode>) -> (UiBindingPublishValidator, Uuid) {
        let realm_id = Uuid::new_v4();
        let theme_id = nodes
            .first()
            .map(|node| node.theme_id)
            .unwrap_or_else(Uuid::new_v4);
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
        let theme_repo = Arc::new(TestThemeRepo::new(theme, Some(binding), nodes));
        let tx_manager = Arc::new(TestTxManager::default());
        let theme_service = Arc::new(ThemeResolverService::new(theme_repo, tx_manager));

        let mut registry = RuntimeRegistry::new();
        registry.register_definition("core.auth.password", StepType::Authenticator);
        registry.register_definition("core.oidc.consent", StepType::Authenticator);
        registry.register_definition("core.ui.no_default", StepType::Authenticator);
        registry.register_definition("core.ui.scripted", StepType::Authenticator);
        let runtime_registry = Arc::new(registry);

        let providers: Vec<Box<dyn NodeProvider>> = vec![
            Box::new(PasswordNodeProvider),
            Box::new(OidcConsentNodeProvider),
            Box::new(NoDefaultUiNodeProvider),
            Box::new(ScriptedUiNodeProvider),
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

    fn build_theme_node(theme_id: Uuid, node_key: &str) -> ThemeNode {
        let blueprint = serde_json::json!([
            {
                "id": "node-1",
                "type": "Text",
                "props": { "text": "Hello" }
            }
        ]);
        ThemeNode {
            id: Uuid::new_v4(),
            theme_id,
            node_key: node_key.to_string(),
            blueprint_json: serde_json::to_string(&blueprint).unwrap(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    fn build_scripted_ui_graph(script: &str) -> Value {
        serde_json::json!({
            "nodes": [
                {
                    "id": "scripted-1",
                    "type": "core.ui.scripted",
                    "position": { "x": 0, "y": 0 },
                    "data": {
                        "label": "Scripted UI",
                        "config": {
                            "template_key": "login",
                            "script": script
                        },
                        "outputs": ["success", "failure"]
                    },
                    "next": {}
                }
            ],
            "edges": []
        })
    }

    fn theme_node(theme_id: Uuid, key: &str, blueprint: Value) -> ThemeNode {
        ThemeNode {
            id: Uuid::new_v4(),
            theme_id,
            node_key: key.to_string(),
            blueprint_json: blueprint.to_string(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
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

    #[tokio::test]
    async fn publish_validator_rejects_invalid_signal_type() {
        let theme_id = Uuid::new_v4();
        let node = theme_node(
            theme_id,
            "custom.signal",
            json!({
                "layout": "default",
                "nodes": [
                    {
                        "type": "Component",
                        "component": "Button",
                        "props": {
                            "label": "Continue",
                            "actions": [
                                {
                                    "trigger": "on_click",
                                    "signal": {
                                        "type": "unknown_signal",
                                        "node_id": "node-1"
                                    }
                                }
                            ]
                        }
                    }
                ]
            }),
        );

        let (validator, realm_id) = build_validator_with_nodes(vec![node]);
        let graph = graph_with_node(
            "core.auth.password",
            json!({ "ui": { "page_key": "custom.signal" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Signal bindings use unsupported types"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_signal_missing_node() {
        let theme_id = Uuid::new_v4();
        let node = theme_node(
            theme_id,
            "custom.signal",
            json!({
                "layout": "default",
                "nodes": [
                    {
                        "type": "Component",
                        "component": "Button",
                        "props": {
                            "label": "Continue",
                            "actions": [
                                {
                                    "trigger": "on_click",
                                    "signal": {
                                        "type": "submit_node",
                                        "node_id": "missing-node"
                                    }
                                }
                            ]
                        }
                    }
                ]
            }),
        );

        let (validator, realm_id) = build_validator_with_nodes(vec![node]);
        let graph = graph_with_node(
            "core.auth.password",
            json!({ "ui": { "page_key": "custom.signal" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Signal bindings reference missing nodes"));
        assert!(message.contains("node_id=missing-node"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_invalid_payload_map() {
        let theme_id = Uuid::new_v4();
        let node = theme_node(
            theme_id,
            "custom.signal",
            json!({
                "layout": "default",
                "nodes": [
                    {
                        "type": "Component",
                        "component": "Button",
                        "props": {
                            "label": "Continue",
                            "actions": [
                                {
                                    "trigger": "on_click",
                                    "signal": {
                                        "type": "submit_node",
                                        "payload_map": {
                                            "email": "inputs."
                                        }
                                    }
                                }
                            ]
                        }
                    }
                ]
            }),
        );

        let (validator, realm_id) = build_validator_with_nodes(vec![node]);
        let graph = graph_with_node(
            "core.auth.password",
            json!({ "ui": { "page_key": "custom.signal" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Signal payload_map invalid"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_unknown_input_binding() {
        let theme_id = Uuid::new_v4();
        let node = theme_node(
            theme_id,
            "custom.signal",
            json!({
                "layout": "default",
                "nodes": [
                    {
                        "type": "Component",
                        "component": "Input",
                        "props": { "name": "email" }
                    },
                    {
                        "type": "Component",
                        "component": "Button",
                        "props": {
                            "label": "Continue",
                            "actions": [
                                {
                                    "trigger": "on_click",
                                    "signal": {
                                        "type": "submit_node",
                                        "payload_map": {
                                            "password": "inputs.password"
                                        }
                                    }
                                }
                            ]
                        }
                    }
                ]
            }),
        );

        let (validator, realm_id) = build_validator_with_nodes(vec![node]);
        let graph = graph_with_node(
            "core.auth.password",
            json!({ "ui": { "page_key": "custom.signal" } }),
        );

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Signal payload_map invalid"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_scripted_ui_unknown_template_key() {
        let theme_id = Uuid::new_v4();
        let nodes = vec![build_theme_node(theme_id, "login")];
        let (validator, realm_id) = build_validator_with_nodes(nodes);
        let script = "return { outcome: 'challenge', context: { template_key: 'missing_page' } };";
        let graph = build_scripted_ui_graph(script);

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Scripted UI validation failed"));
        assert!(message.contains("unknown template_key"));
    }

    #[tokio::test]
    async fn publish_validator_rejects_scripted_ui_invalid_patch() {
        let theme_id = Uuid::new_v4();
        let nodes = vec![build_theme_node(theme_id, "login")];
        let (validator, realm_id) = build_validator_with_nodes(nodes);
        let script = "return { outcome: 'challenge', context: { template_key: 'login', ui_patch: [{ id: 'x', type: 'Unknown' }] } };";
        let graph = build_scripted_ui_graph(script);

        let err = validator.validate(realm_id, &graph).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Scripted UI validation failed"));
        assert!(message.contains("Unsupported node type"));
    }
}
