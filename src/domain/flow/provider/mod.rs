use crate::domain::flow::models::NodeCapabilities;
use crate::domain::ui::{PageCategory, UiSurface};
use serde_json::Value;

/// The contract every Node Type must fulfill.
pub trait NodeProvider: Send + Sync {
    /// Unique ID (e.g., "core.auth.password")
    fn id(&self) -> &'static str;

    /// User-friendly name (e.g., "Username & Password")
    fn display_name(&self) -> &'static str;

    /// Description for the toolbox
    fn description(&self) -> &'static str;

    /// The Icon key (mapped in frontend)
    fn icon(&self) -> &'static str;

    /// Category (Start, Authenticator, Logic, Terminal)
    fn category(&self) -> &'static str;

    /// Versioned Node Contract
    fn contract_version(&self) -> &'static str {
        "1"
    }

    /// Inputs required by this node (e.g., "in")
    fn inputs(&self) -> Vec<&'static str> {
        vec!["in"]
    }

    /// Outputs produced by this node (e.g., "success", "failure")
    fn outputs(&self) -> Vec<&'static str>;

    /// The JSON Schema for configuration.
    /// The Frontend will use this to auto-generate the settings form.
    fn config_schema(&self) -> Value;

    /// Whether this node renders a user-facing screen at runtime.
    fn supports_ui(&self) -> bool {
        false
    }

    /// Default Fluid page key for UI-capable nodes.
    fn default_template_key(&self) -> Option<&'static str> {
        None
    }

    /// UI surface used for rendering (e.g., form vs awaiting action).
    fn ui_surface(&self) -> Option<UiSurface> {
        None
    }

    /// Allowed theme page categories for UI-capable nodes.
    fn allowed_page_categories(&self) -> Vec<PageCategory> {
        Vec::new()
    }

    /// Whether this node can suspend asynchronously.
    fn async_pause(&self) -> bool {
        false
    }

    /// Whether this node can trigger side effects (email, audit, etc).
    fn side_effects(&self) -> bool {
        false
    }

    /// Whether this node needs access to secrets.
    fn requires_secrets(&self) -> bool {
        false
    }

    /// Capability model (shared contract between UI + backend).
    fn capabilities(&self) -> NodeCapabilities {
        NodeCapabilities {
            supports_ui: self.supports_ui(),
            ui_surface: self.ui_surface(),
            allowed_page_categories: self.allowed_page_categories(),
            async_pause: self.async_pause(),
            side_effects: self.side_effects(),
            requires_secrets: self.requires_secrets(),
        }
    }

    /// (Optional) Default configuration values
    fn default_config(&self) -> Value {
        serde_json::json!({})
    }
}

#[cfg(test)]
mod tests;
