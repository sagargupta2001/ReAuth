use serde_json::{json, Value};

pub struct FlowTemplates;

impl FlowTemplates {
    pub fn browser_flow() -> Value {
        json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" },
                    "next": { "default": "auth-cookie" }
                },
                {
                    "id": "auth-cookie",
                    "type": "core.auth.cookie",
                    "position": { "x": 250, "y": 100 },
                    "data": {
                        "label": "Check SSO Cookie",
                        "config": {
                            "auth_type": "core.auth.cookie"
                        },
                        "outputs": ["continue"]
                    },
                    "next": { "continue": "auth-password" }
                },
                {
                    "id": "auth-password",
                    "type": "core.auth.password",
                    "position": { "x": 250, "y": 250 },
                    "data": {
                        "label": "Username & Password",
                        "config": { "auth_type": "core.auth.password", "max_attempts": 3 },
                        "outputs": ["success", "failure"]
                    },
                    "next": { "success": "success" }
                },
                {
                    "id": "success",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 400 },
                    "data": { "label": "Allow Access" },
                    "next": {}
                }
            ],
            "edges": [
                // Start -> Cookie
                { "id": "e1", "source": "start", "target": "auth-cookie" },

                // [FIX] Cookie -> Password
                // We use sourceHandle: "continue" to match the NodeProvider
                {
                    "id": "e2",
                    "source": "auth-cookie",
                    "sourceHandle": "continue",
                    "target": "auth-password"
                },

                // Password -> Success
                { "id": "e3", "source": "auth-password", "sourceHandle": "success", "target": "success" }
            ]
        })
    }
    pub fn direct_grant_flow() -> Value {
        // ... (Keep existing implementation)
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

    pub fn reset_credentials_flow() -> Value {
        // ... (Keep existing implementation)
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

    pub fn registration_flow() -> Value {
        // ... (Keep existing implementation)
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
