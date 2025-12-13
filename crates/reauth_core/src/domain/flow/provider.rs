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

    /// Inputs required by this node (e.g., "in")
    fn inputs(&self) -> Vec<&'static str> { vec!["in"] }

    /// Outputs produced by this node (e.g., "success", "failure")
    fn outputs(&self) -> Vec<&'static str>;

    /// The JSON Schema for configuration.
    /// The Frontend will use this to auto-generate the settings form.
    fn config_schema(&self) -> Value;

    /// (Optional) Default configuration values
    fn default_config(&self) -> Value { serde_json::json!({}) }
}