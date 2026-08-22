use serde_json::{json, Value};

pub struct FlowTemplates;

impl FlowTemplates {
    pub fn browser_flow() -> Value {
        json!({"edges":[{"id":"e1","source":"start","target":"auth-cookie"},{"id":"e2","source":"auth-cookie","sourceHandle":"continue","target":"condition-sso"},{"id":"e3","source":"condition-sso","sourceHandle":"false","target":"auth-password"},{"id":"e4","selected":false,"source":"condition-sso","sourceHandle":"true","target":"condition-oidc"},{"id":"e5","source":"auth-password","sourceHandle":"success","target":"condition-oidc"},{"id":"e6","source":"auth-password","sourceHandle":"force_reset","target":"auth-force-reset"},{"id":"e7","source":"auth-force-reset","sourceHandle":"success","target":"condition-oidc"},{"id":"e8","source":"condition-oidc","sourceHandle":"true","target":"oidc-consent"},{"id":"e9","source":"condition-oidc","sourceHandle":"false","target":"success"},{"id":"e10","source":"oidc-consent","sourceHandle":"allow","target":"success"},{"id":"e11","source":"oidc-consent","sourceHandle":"deny","target":"deny"}],"nodes":[{"data":{"label":"Start"},"id":"start","measured":{"height":60,"width":151},"next":{"default":"auth-cookie"},"position":{"x":250,"y":0},"type":"core.start"},{"data":{"config":{"auth_type":"core.auth.cookie"},"label":"Check SSO Cookie","outputs":["continue"]},"dragging":false,"id":"auth-cookie","measured":{"height":89,"width":256},"next":{"continue":"condition-sso"},"position":{"x":197.4059003051882,"y":120},"selected":false,"type":"core.auth.cookie"},{"data":{"config":{"context_path":"user_id","logic_type":"core.logic.condition","operator":"exists"},"label":"SSO Session?","outputs":["true","false"]},"dragging":false,"id":"condition-sso","measured":{"height":89,"width":256},"next":{"false":"auth-password","true":"condition-oidc"},"position":{"x":194.5925646327236,"y":267.53918406110415},"selected":false,"type":"core.logic.condition"},{"data":{"config":{"auth_type":"core.auth.password","max_attempts":3,"template_key":"login"},"label":"Username & Password","outputs":["success","force_reset","failure"]},"dragging":false,"id":"auth-password","measured":{"height":89,"width":256},"next":{"force_reset":"auth-force-reset","success":"condition-oidc"},"position":{"x":340.1546046318235,"y":478.8389468034793},"selected":false,"type":"core.auth.password"},{"data":{"config":{"auth_type":"core.auth.reset_password","template_key":"reset_password"},"label":"Force Password Reset","outputs":["success","failure"]},"dragging":false,"id":"auth-force-reset","measured":{"height":89,"width":256},"next":{"success":"condition-oidc"},"position":{"x":358.59237341189026,"y":758.6772542846306},"selected":false,"type":"core.auth.reset_password"},{"data":{"config":{"context_path":"oidc.client_id","logic_type":"core.logic.condition","operator":"exists"},"label":"OIDC Consent Required?","outputs":["true","false"]},"dragging":false,"id":"condition-oidc","measured":{"height":89,"width":256},"next":{"false":"success","true":"oidc-consent"},"position":{"x":-14.518366142443044,"y":699.3793916664725},"selected":true,"type":"core.logic.condition"},{"data":{"config":{"auth_type":"core.oidc.consent","template_key":"consent"},"label":"OIDC Consent","outputs":["allow","deny"]},"dragging":false,"id":"oidc-consent","measured":{"height":89,"width":256},"next":{"allow":"success","deny":"deny"},"position":{"x":-174.9096991518508,"y":935.373021256728},"selected":false,"type":"core.oidc.consent"},{"data":{"label":"Allow Access"},"dragging":false,"id":"success","measured":{"height":60,"width":162},"next":{},"position":{"x":85.09456913868655,"y":1360.0465340915562},"selected":false,"type":"core.terminal.allow"},{"data":{"config":{"is_failure":true},"label":"Deny Access"},"dragging":false,"id":"deny","measured":{"height":60,"width":159},"next":{},"position":{"x":-73.16050345958413,"y":1107.630195974206},"selected":false,"type":"core.terminal.deny"}],"viewport":{"x":261.92684726251605,"y":-173.9793678635525,"zoom":0.695396737225537}})
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
        json!({"edges":[{"id":"e1","source":"auth-password","sourceHandle":"success","target":"allow"}],"nodes":[{"data":{"config":{"auth_type":"core.auth.password","template_key":"login"},"label":"Username & Password","outputs":["success","failure"]},"id":"auth-password","measured":{"height":89,"width":256},"position":{"x":250,"y":50},"type":"core.auth.password"},{"data":{"label":"Allow Access"},"dragging":false,"id":"allow","measured":{"height":60,"width":162},"position":{"x":237.5,"y":199},"selected":true,"type":"core.terminal.allow"}],"viewport":{"x":-156,"y":230.5,"zoom":2}})
    }

    pub fn reset_credentials_flow() -> Value {
        json!({"edges":[{"id":"e0","source":"start","target":"auth-forgot"},{"id":"e1","source":"auth-forgot","sourceHandle":"success","target":"recovery-issue"},{"id":"e2","source":"recovery-issue","sourceHandle":"issued","target":"reset-password"},{"id":"e3","source":"reset-password","sourceHandle":"success","target":"allow"}],"nodes":[{"data":{"label":"Start"},"id":"start","measured":{"height":60,"width":151},"next":{"default":"auth-forgot"},"position":{"x":250,"y":0},"type":"core.start"},{"data":{"config":{"auth_type":"core.auth.forgot_credentials","template_key":"forgot_credentials"},"label":"Forgot Credentials","outputs":["success","failure"]},"dragging":false,"id":"auth-forgot","measured":{"height":89,"width":256},"position":{"x":196.89725330620547,"y":118.22990844354015},"selected":false,"type":"core.auth.forgot_credentials"},{"data":{"config":{"logic_type":"core.logic.recovery_issue"},"label":"Issue Recovery Token","outputs":["issued"]},"dragging":false,"id":"recovery-issue","measured":{"height":89,"width":256},"position":{"x":137.8942014242116,"y":278.91149542217704},"selected":false,"type":"core.logic.recovery_issue"},{"data":{"config":{"auth_type":"core.auth.reset_password","template_key":"reset_password"},"label":"Reset Password","outputs":["success","failure"]},"dragging":false,"id":"reset-password","measured":{"height":89,"width":256},"position":{"x":135.05729448202993,"y":434.1195033669032},"selected":false,"type":"core.auth.reset_password"},{"data":{"label":"Allow Access"},"dragging":false,"id":"allow","measured":{"height":60,"width":162},"position":{"x":123.0638118376398,"y":593.9122108287161},"selected":true,"type":"core.terminal.allow"}],"viewport":{"x":66.54373118581088,"y":-40.92715558715645,"zoom":1.2447199044444837}})
    }

    pub fn registration_flow() -> Value {
        json!({"edges":[{"id":"e0","source":"start","target":"auth-register"},{"id":"e1","source":"auth-register","sourceHandle":"success","target":"allow"}],"nodes":[{"data":{"label":"Start"},"id":"start","measured":{"height":60,"width":151},"next":{"default":"auth-register"},"position":{"x":250,"y":0},"type":"core.start"},{"data":{"config":{"allow_when_invited":true,"auth_type":"core.auth.register","template_key":"register"},"label":"Register Account","outputs":["success","failure"]},"dragging":false,"id":"auth-register","measured":{"height":89,"width":256},"position":{"x":197,"y":113},"selected":false,"type":"core.auth.register"},{"data":{"label":"Allow Access"},"dragging":false,"id":"allow","measured":{"height":60,"width":162},"position":{"x":184.5,"y":262.5},"selected":true,"type":"core.terminal.allow"}],"viewport":{"x":-156,"y":210.5,"zoom":2}})
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
        json!({"edges":[{"id":"e0","source":"start","target":"invitation-validate"},{"id":"e1","source":"invitation-validate","sourceHandle":"valid","target":"invitation-issue"},{"id":"e2","source":"invitation-issue","sourceHandle":"issued","target":"auth-register"},{"id":"e3","source":"auth-register","sourceHandle":"success","target":"allow"},{"id":"e4","source":"invitation-validate","sourceHandle":"expired","target":"invitation-unavailable"},{"id":"e5","source":"invitation-validate","sourceHandle":"consumed","target":"invitation-unavailable"},{"id":"e6","source":"invitation-validate","sourceHandle":"invalid","target":"invitation-unavailable"},{"id":"e7","source":"invitation-unavailable","sourceHandle":"failure","target":"deny"}],"nodes":[{"data":{"label":"Start"},"id":"start","measured":{"height":60,"width":151},"next":{"default":"invitation-validate"},"position":{"x":250,"y":0},"type":"core.start"},{"data":{"config":{"logic_type":"core.logic.invitation_token"},"label":"Validate Invitation","outputs":["valid","expired","consumed","invalid"]},"dragging":false,"id":"invitation-validate","measured":{"height":89,"width":256},"next":{"consumed":"invitation-unavailable","expired":"invitation-unavailable","invalid":"invitation-unavailable","valid":"invitation-issue"},"position":{"x":197.26347914547307,"y":118.0467955239064},"selected":false,"type":"core.logic.invitation_token"},{"data":{"config":{"logic_type":"core.logic.issue_invitation","resend_path":"/invite/accept","resume_node_id":"auth-register","resume_path":"/invite/accept"},"label":"Issue Invitation Token","outputs":["issued"]},"dragging":false,"id":"invitation-issue","measured":{"height":89,"width":256},"next":{"issued":"auth-register"},"position":{"x":-80.74262461851477,"y":378.4944048830112},"selected":false,"type":"core.logic.issue_invitation"},{"data":{"config":{"allow_when_invited":true,"auth_type":"core.auth.register","template_key":"register"},"label":"Register Account","outputs":["success","failure"]},"dragging":false,"id":"auth-register","measured":{"height":89,"width":256},"position":{"x":-83.95233474607933,"y":547.0045650724895},"selected":false,"type":"core.auth.register"},{"data":{"config":{"auth_type":"core.auth.invitation_unavailable","template_key":"invitation_unavailable","title":"Invitation Link Unavailable"},"label":"Invitation Unavailable","outputs":["failure"]},"dragging":false,"id":"invitation-unavailable","measured":{"height":89,"width":256},"next":{"failure":"deny"},"position":{"x":222.74669379450657,"y":378.3112919633774},"selected":false,"type":"core.auth.invitation_unavailable"},{"data":{"label":"Allow Access"},"dragging":false,"id":"allow","measured":{"height":60,"width":162},"position":{"x":-96.0057127462573,"y":734.2725245922331},"selected":true,"type":"core.terminal.allow"},{"data":{"config":{"error_code":"invitation_unavailable","error_message":"Invitation unavailable"},"label":"Deny Access"},"dragging":false,"id":"deny","measured":{"height":60,"width":159},"position":{"x":268.7686666111528,"y":539.7787892213412},"selected":false,"type":"core.terminal.deny"}],"viewport":{"x":245.70935229707524,"y":162.46106123234802,"zoom":0.7703111342379536}})
    }
}
