// crates/application/src/flow_manager/templates.rs

use crate::domain::flow::provider::NodeProvider;
use serde_json::{json, Value};

pub struct FlowTemplates;

impl FlowTemplates {
    /// Helper to get IDs dynamically (optional, but safer)
    fn id<T: NodeProvider + Default>() -> String {
        T::default().id().to_string()
    }

    pub fn browser_flow() -> Value {
        // We use the `json!` macro which is much cleaner than raw strings
        // and supports trailing commas / comments.
        json!({
            "nodes": [
                {
                    "id": "start",
                    // We map the type directly to the ID defined in your Structs
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" }
                },
                {
                    "id": "auth-password",
                    "type": "core.auth.password",
                    "position": { "x": 250, "y": 150 },
                    "data": {
                        "label": "Username & Password",
                        "config": {
                            // Defaults are handled by the node, but we can override here
                            "max_attempts": 3
                        }
                    }
                },
                {
                    "id": "success",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 300 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                // Start -> Password
                { "id": "e1", "source": "start", "target": "auth-password" },
                // Password (Success) -> Allow
                { "id": "e2", "source": "auth-password", "sourceHandle": "success", "target": "success" }
            ]
        })
    }

    pub fn direct_grant_flow() -> Value {
        json!({
            "nodes": [
                {
                    "id": "auth-password",
                    "type": "core.auth.password",
                    "position": { "x": 250, "y": 50 },
                    "data": { "label": "Direct Grant Auth" }
                },
                {
                    "id": "success",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 200 },
                    "data": { "label": "Success" }
                }
            ],
            "edges": [
                { "id": "e1", "source": "auth-password", "sourceHandle": "success", "target": "success" }
            ]
        })
    }
}
