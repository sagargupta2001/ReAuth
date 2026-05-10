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
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Check SSO Cookie",
                        "config": {
                            "auth_type": "core.auth.cookie"
                        },
                        "outputs": ["continue"]
                    },
                    "next": { "continue": "condition-sso" }
                },
                {
                    "id": "condition-sso",
                    "type": "core.logic.condition",
                    "position": { "x": 250, "y": 260 },
                    "data": {
                        "label": "SSO Session?",
                        "config": {
                            "logic_type": "core.logic.condition",
                            "context_path": "user_id",
                            "operator": "exists"
                        },
                        "outputs": ["true", "false"]
                    },
                    "next": { "true": "condition-oidc", "false": "auth-password" }
                },
                {
                    "id": "auth-password",
                    "type": "core.auth.password",
                    "position": { "x": 250, "y": 420 },
                    "data": {
                        "label": "Username & Password",
                        "config": {
                            "auth_type": "core.auth.password",
                            "template_key": "login",
                            "max_attempts": 3
                        },
                        "outputs": ["success", "force_reset", "failure"]
                    },
                    "next": { "success": "condition-oidc", "force_reset": "auth-force-reset" }
                },
                {
                    "id": "auth-force-reset",
                    "type": "core.auth.reset_password",
                    "position": { "x": 520, "y": 420 },
                    "data": {
                        "label": "Force Password Reset",
                        "config": {
                            "auth_type": "core.auth.reset_password",
                            "template_key": "reset_password"
                        },
                        "outputs": ["success", "failure"]
                    },
                    "next": { "success": "condition-oidc" }
                },
                {
                    "id": "condition-oidc",
                    "type": "core.logic.condition",
                    "position": { "x": 250, "y": 580 },
                    "data": {
                        "label": "OIDC Consent Required?",
                        "config": {
                            "logic_type": "core.logic.condition",
                            "context_path": "oidc.client_id",
                            "operator": "exists"
                        },
                        "outputs": ["true", "false"]
                    },
                    "next": { "true": "oidc-consent", "false": "success" }
                },
                {
                    "id": "oidc-consent",
                    "type": "core.oidc.consent",
                    "position": { "x": 250, "y": 730 },
                    "data": {
                        "label": "OIDC Consent",
                        "config": {
                            "auth_type": "core.oidc.consent",
                            "template_key": "consent"
                        },
                        "outputs": ["allow", "deny"]
                    },
                    "next": { "allow": "success", "deny": "deny" }
                },
                {
                    "id": "success",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 880 },
                    "data": { "label": "Allow Access" },
                    "next": {}
                },
                {
                    "id": "deny",
                    "type": "core.terminal.deny",
                    "position": { "x": 460, "y": 880 },
                    "data": {
                        "label": "Deny Access",
                        "config": { "is_failure": true }
                    },
                    "next": {}
                }
            ],
            "edges": [
                { "id": "e1", "source": "start", "target": "auth-cookie" },
                {
                    "id": "e2",
                    "source": "auth-cookie",
                    "sourceHandle": "continue",
                    "target": "condition-sso"
                },
                {
                    "id": "e3",
                    "source": "condition-sso",
                    "sourceHandle": "false",
                    "target": "auth-password"
                },
                {
                    "id": "e4",
                    "source": "condition-sso",
                    "sourceHandle": "true",
                    "target": "condition-oidc"
                },
                {
                    "id": "e5",
                    "source": "auth-password",
                    "sourceHandle": "success",
                    "target": "condition-oidc"
                },
                {
                    "id": "e6",
                    "source": "auth-password",
                    "sourceHandle": "force_reset",
                    "target": "auth-force-reset"
                },
                {
                    "id": "e7",
                    "source": "auth-force-reset",
                    "sourceHandle": "success",
                    "target": "condition-oidc"
                },
                {
                    "id": "e8",
                    "source": "condition-oidc",
                    "sourceHandle": "true",
                    "target": "oidc-consent"
                },
                {
                    "id": "e9",
                    "source": "condition-oidc",
                    "sourceHandle": "false",
                    "target": "success"
                },
                {
                    "id": "e10",
                    "source": "oidc-consent",
                    "sourceHandle": "allow",
                    "target": "success"
                },
                {
                    "id": "e11",
                    "source": "oidc-consent",
                    "sourceHandle": "deny",
                    "target": "deny"
                }
            ]
        })
    }

    pub fn passkey_first_browser_flow() -> Value {
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
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Check SSO Cookie",
                        "config": {
                            "auth_type": "core.auth.cookie"
                        },
                        "outputs": ["continue"]
                    },
                    "next": { "continue": "condition-sso" }
                },
                {
                    "id": "condition-sso",
                    "type": "core.logic.condition",
                    "position": { "x": 250, "y": 260 },
                    "data": {
                        "label": "SSO Session?",
                        "config": {
                            "logic_type": "core.logic.condition",
                            "context_path": "user_id",
                            "operator": "exists"
                        },
                        "outputs": ["true", "false"]
                    },
                    "next": { "true": "condition-oidc", "false": "auth-passkey" }
                },
                {
                    "id": "auth-passkey",
                    "type": "core.auth.passkey_assert",
                    "position": { "x": 250, "y": 420 },
                    "data": {
                        "label": "Passkey Sign In",
                        "config": {
                            "auth_type": "core.auth.passkey_assert",
                            "template_key": "passkey_assert",
                            "intent": "login"
                        },
                        "outputs": ["success", "fallback", "failure"]
                    },
                    "next": { "success": "condition-oidc", "fallback": "auth-password", "failure": "auth-password" }
                },
                {
                    "id": "auth-password",
                    "type": "core.auth.password",
                    "position": { "x": 520, "y": 420 },
                    "data": {
                        "label": "Username & Password",
                        "config": {
                            "auth_type": "core.auth.password",
                            "template_key": "login",
                            "max_attempts": 3
                        },
                        "outputs": ["success", "force_reset", "failure"]
                    },
                    "next": { "success": "condition-oidc", "force_reset": "auth-force-reset" }
                },
                {
                    "id": "auth-force-reset",
                    "type": "core.auth.reset_password",
                    "position": { "x": 790, "y": 420 },
                    "data": {
                        "label": "Force Password Reset",
                        "config": {
                            "auth_type": "core.auth.reset_password",
                            "template_key": "reset_password"
                        },
                        "outputs": ["success", "failure"]
                    },
                    "next": { "success": "condition-oidc" }
                },
                {
                    "id": "condition-oidc",
                    "type": "core.logic.condition",
                    "position": { "x": 250, "y": 580 },
                    "data": {
                        "label": "OIDC Consent Required?",
                        "config": {
                            "logic_type": "core.logic.condition",
                            "context_path": "oidc.client_id",
                            "operator": "exists"
                        },
                        "outputs": ["true", "false"]
                    },
                    "next": { "true": "oidc-consent", "false": "success" }
                },
                {
                    "id": "oidc-consent",
                    "type": "core.oidc.consent",
                    "position": { "x": 250, "y": 730 },
                    "data": {
                        "label": "OIDC Consent",
                        "config": {
                            "auth_type": "core.oidc.consent",
                            "template_key": "consent"
                        },
                        "outputs": ["allow", "deny"]
                    },
                    "next": { "allow": "success", "deny": "deny" }
                },
                {
                    "id": "success",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 880 },
                    "data": { "label": "Allow Access" },
                    "next": {}
                },
                {
                    "id": "deny",
                    "type": "core.terminal.deny",
                    "position": { "x": 460, "y": 880 },
                    "data": {
                        "label": "Deny Access",
                        "config": { "is_failure": true }
                    },
                    "next": {}
                }
            ],
            "edges": [
                { "id": "e1", "source": "start", "target": "auth-cookie" },
                {
                    "id": "e2",
                    "source": "auth-cookie",
                    "sourceHandle": "continue",
                    "target": "condition-sso"
                },
                {
                    "id": "e3",
                    "source": "condition-sso",
                    "sourceHandle": "false",
                    "target": "auth-passkey"
                },
                {
                    "id": "e4",
                    "source": "condition-sso",
                    "sourceHandle": "true",
                    "target": "condition-oidc"
                },
                {
                    "id": "e5",
                    "source": "auth-passkey",
                    "sourceHandle": "success",
                    "target": "condition-oidc"
                },
                {
                    "id": "e6",
                    "source": "auth-passkey",
                    "sourceHandle": "fallback",
                    "target": "auth-password"
                },
                {
                    "id": "e7",
                    "source": "auth-passkey",
                    "sourceHandle": "failure",
                    "target": "auth-password"
                },
                {
                    "id": "e8",
                    "source": "auth-password",
                    "sourceHandle": "success",
                    "target": "condition-oidc"
                },
                {
                    "id": "e9",
                    "source": "auth-password",
                    "sourceHandle": "force_reset",
                    "target": "auth-force-reset"
                },
                {
                    "id": "e10",
                    "source": "auth-force-reset",
                    "sourceHandle": "success",
                    "target": "condition-oidc"
                },
                {
                    "id": "e11",
                    "source": "condition-oidc",
                    "sourceHandle": "true",
                    "target": "oidc-consent"
                },
                {
                    "id": "e12",
                    "source": "condition-oidc",
                    "sourceHandle": "false",
                    "target": "success"
                },
                {
                    "id": "e13",
                    "source": "oidc-consent",
                    "sourceHandle": "allow",
                    "target": "success"
                },
                {
                    "id": "e14",
                    "source": "oidc-consent",
                    "sourceHandle": "deny",
                    "target": "deny"
                }
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
                    "data": {
                        "label": "Username & Password",
                        "config": {
                            "auth_type": "core.auth.password",
                            "template_key": "login"
                        },
                        "outputs": ["success", "failure"]
                    }
                },
                {
                    "id": "allow",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 200 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                { "id": "e1", "source": "auth-password", "sourceHandle": "success", "target": "allow" }
            ]
        })
    }

    pub fn reset_credentials_flow() -> Value {
        // ... (Keep existing implementation)
        json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" },
                    "next": { "default": "auth-forgot" }
                },
                {
                    "id": "auth-forgot",
                    "type": "core.auth.forgot_credentials",
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Forgot Credentials",
                        "config": {
                            "auth_type": "core.auth.forgot_credentials",
                            "template_key": "forgot_credentials"
                        },
                        "outputs": ["success", "failure"]
                    }
                },
                {
                    "id": "recovery-issue",
                    "type": "core.logic.recovery_issue",
                    "position": { "x": 250, "y": 250 },
                    "data": {
                        "label": "Issue Recovery Token",
                        "config": {
                            "logic_type": "core.logic.recovery_issue"
                        },
                        "outputs": ["issued"]
                    }
                },
                {
                    "id": "reset-password",
                    "type": "core.auth.reset_password",
                    "position": { "x": 250, "y": 380 },
                    "data": {
                        "label": "Reset Password",
                        "config": {
                            "auth_type": "core.auth.reset_password",
                            "template_key": "reset_password"
                        },
                        "outputs": ["success", "failure"]
                    }
                },
                {
                    "id": "allow",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 520 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                { "id": "e0", "source": "start", "target": "auth-forgot" },
                { "id": "e1", "source": "auth-forgot", "sourceHandle": "success", "target": "recovery-issue" },
                { "id": "e2", "source": "recovery-issue", "sourceHandle": "issued", "target": "reset-password" },
                { "id": "e3", "source": "reset-password", "sourceHandle": "success", "target": "allow" }
            ]
        })
    }

    pub fn registration_flow() -> Value {
        // ... (Keep existing implementation)
        json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" },
                    "next": { "default": "auth-register" }
                },
                {
                    "id": "auth-register",
                    "type": "core.auth.register",
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Register Account",
                        "config": {
                            "auth_type": "core.auth.register",
                            "template_key": "register",
                            "allow_when_invited": true
                        },
                        "outputs": ["success", "failure"]
                    }
                },
                {
                    "id": "allow",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 270 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                { "id": "e0", "source": "start", "target": "auth-register" },
                { "id": "e1", "source": "auth-register", "sourceHandle": "success", "target": "allow" }
            ]
        })
    }

    pub fn passkey_enroll_registration_flow() -> Value {
        json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" },
                    "next": { "default": "auth-register" }
                },
                {
                    "id": "auth-register",
                    "type": "core.auth.register",
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Register Account",
                        "config": {
                            "auth_type": "core.auth.register",
                            "template_key": "register"
                        },
                        "outputs": ["success", "failure"]
                    },
                    "next": { "success": "passkey-enroll" }
                },
                {
                    "id": "passkey-enroll",
                    "type": "core.auth.passkey_enroll",
                    "position": { "x": 250, "y": 280 },
                    "data": {
                        "label": "Passkey Enroll",
                        "config": {
                            "auth_type": "core.auth.passkey_enroll",
                            "template_key": "passkey_enroll",
                            "allow_skip": true
                        },
                        "outputs": ["success", "skip", "failure"]
                    },
                    "next": { "success": "allow", "skip": "allow", "failure": "allow" }
                },
                {
                    "id": "allow",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 450 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                { "id": "e0", "source": "start", "target": "auth-register" },
                { "id": "e1", "source": "auth-register", "sourceHandle": "success", "target": "passkey-enroll" },
                { "id": "e2", "source": "passkey-enroll", "sourceHandle": "success", "target": "allow" },
                { "id": "e3", "source": "passkey-enroll", "sourceHandle": "skip", "target": "allow" },
                { "id": "e4", "source": "passkey-enroll", "sourceHandle": "failure", "target": "allow" }
            ]
        })
    }

    pub fn invitation_flow() -> Value {
        json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "core.start",
                    "position": { "x": 250, "y": 0 },
                    "data": { "label": "Start" },
                    "next": { "default": "invitation-validate" }
                },
                {
                    "id": "invitation-validate",
                    "type": "core.logic.invitation_token",
                    "position": { "x": 250, "y": 120 },
                    "data": {
                        "label": "Validate Invitation",
                        "config": {
                            "logic_type": "core.logic.invitation_token"
                        },
                        "outputs": ["valid"]
                    },
                    "next": { "valid": "invitation-issue" }
                },
                {
                    "id": "invitation-issue",
                    "type": "core.logic.issue_invitation",
                    "position": { "x": 250, "y": 260 },
                    "data": {
                        "label": "Issue Invitation Token",
                        "config": {
                            "logic_type": "core.logic.issue_invitation",
                            "resume_path": "/invite/accept",
                            "resend_path": "/invite/accept",
                            "resume_node_id": "auth-register"
                        },
                        "outputs": ["issued"]
                    },
                    "next": { "issued": "auth-register" }
                },
                {
                    "id": "auth-register",
                    "type": "core.auth.register",
                    "position": { "x": 250, "y": 430 },
                    "data": {
                        "label": "Register Account",
                        "config": {
                            "auth_type": "core.auth.register",
                            "template_key": "register",
                            "allow_when_invited": true
                        },
                        "outputs": ["success", "failure"]
                    }
                },
                {
                    "id": "allow",
                    "type": "core.terminal.allow",
                    "position": { "x": 250, "y": 580 },
                    "data": { "label": "Allow Access" }
                }
            ],
            "edges": [
                { "id": "e0", "source": "start", "target": "invitation-validate" },
                { "id": "e1", "source": "invitation-validate", "sourceHandle": "valid", "target": "invitation-issue" },
                { "id": "e2", "source": "invitation-issue", "sourceHandle": "issued", "target": "auth-register" },
                { "id": "e3", "source": "auth-register", "sourceHandle": "success", "target": "allow" }
            ]
        })
    }
}
