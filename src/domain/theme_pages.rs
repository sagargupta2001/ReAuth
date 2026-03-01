use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct ThemePageTemplate {
    pub key: String,
    pub label: String,
    pub description: String,
    pub blueprint: Value,
}

struct ThemePageDefinition {
    key: &'static str,
    label: &'static str,
    description: &'static str,
}

const SYSTEM_PAGES: &[ThemePageDefinition] = &[
    ThemePageDefinition {
        key: "login",
        label: "Login",
        description: "Primary sign-in screen.",
    },
    ThemePageDefinition {
        key: "register",
        label: "Register",
        description: "Account creation form.",
    },
    ThemePageDefinition {
        key: "forgot_credentials",
        label: "Forgot Credentials",
        description: "Password reset entry.",
    },
    ThemePageDefinition {
        key: "verify_email",
        label: "Verify Email",
        description: "Email verification notice.",
    },
    ThemePageDefinition {
        key: "mfa",
        label: "Multi-Factor",
        description: "OTP and challenge prompts.",
    },
    ThemePageDefinition {
        key: "consent",
        label: "Consent",
        description: "OIDC consent step.",
    },
    ThemePageDefinition {
        key: "magic_link_sent",
        label: "Magic Link Sent",
        description: "Magic link confirmation.",
    },
    ThemePageDefinition {
        key: "error",
        label: "Error",
        description: "Fallback error page.",
    },
];

pub fn system_pages() -> Vec<ThemePageTemplate> {
    SYSTEM_PAGES
        .iter()
        .map(|page| ThemePageTemplate {
            key: page.key.to_string(),
            label: page.label.to_string(),
            description: page.description.to_string(),
            blueprint: default_page_blueprint(page.key).unwrap_or_else(default_fallback_blueprint),
        })
        .collect()
}

pub fn is_valid_page(key: &str) -> bool {
    SYSTEM_PAGES.iter().any(|page| page.key == key) || is_custom_page(key)
}

pub fn is_custom_page(key: &str) -> bool {
    key.starts_with("custom.") && key.len() > "custom.".len()
}

pub fn custom_page_template(key: &str, blueprint: Value) -> ThemePageTemplate {
    ThemePageTemplate {
        key: key.to_string(),
        label: custom_page_label(key),
        description: "Custom page".to_string(),
        blueprint,
    }
}

fn custom_page_label(key: &str) -> String {
    let trimmed = key.strip_prefix("custom.").unwrap_or(key);
    let mut label = String::new();
    for (index, part) in trimmed.split(['-', '_']).enumerate() {
        if part.is_empty() {
            continue;
        }
        if index > 0 && !label.ends_with(' ') {
            label.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            label.extend(first.to_uppercase());
            label.push_str(chars.as_str());
        }
    }
    if label.is_empty() {
        "Custom Page".to_string()
    } else {
        label
    }
}

pub fn default_page_blueprint(key: &str) -> Option<Value> {
    match key {
        "login" => Some(default_login_blueprint()),
        "register" => Some(default_register_blueprint()),
        "forgot_credentials" => Some(default_forgot_blueprint()),
        "verify_email" => Some(default_verify_blueprint()),
        "mfa" => Some(default_mfa_blueprint()),
        "consent" => Some(default_consent_blueprint()),
        "magic_link_sent" => Some(default_magic_link_blueprint()),
        "error" => Some(default_error_blueprint()),
        _ => None,
    }
}

pub fn default_page_blueprint_fallback() -> Value {
    default_fallback_blueprint()
}

fn default_fallback_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            {
                "block": "text",
                "props": { "text": "Page content" },
                "children": []
            }
        ]
    })
}

fn default_login_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Welcome back" }, "children": [] },
            { "block": "input", "props": { "label": "Email or username", "name": "username", "input_type": "text" }, "children": [] },
            { "block": "input", "props": { "label": "Password", "name": "password", "input_type": "password" }, "children": [] },
            { "block": "button", "props": { "label": "Continue", "variant": "primary" }, "children": [] }
        ]
    })
}

fn default_register_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Create your account" }, "children": [] },
            { "block": "input", "props": { "label": "Email", "name": "email", "input_type": "email" }, "children": [] },
            { "block": "input", "props": { "label": "Password", "name": "password", "input_type": "password" }, "children": [] },
            { "block": "button", "props": { "label": "Sign up", "variant": "primary" }, "children": [] }
        ]
    })
}

fn default_forgot_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Reset your password" }, "children": [] },
            { "block": "input", "props": { "label": "Email", "name": "email", "input_type": "email" }, "children": [] },
            { "block": "button", "props": { "label": "Send reset link", "variant": "primary" }, "children": [] }
        ]
    })
}

fn default_verify_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Check your inbox to verify your email." }, "children": [] },
            { "block": "button", "props": { "label": "Resend email", "variant": "secondary" }, "children": [] }
        ]
    })
}

fn default_mfa_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Enter your verification code" }, "children": [] },
            { "block": "input", "props": { "label": "Code", "name": "otp" }, "children": [] },
            { "block": "button", "props": { "label": "Verify", "variant": "primary" }, "children": [] }
        ]
    })
}

fn default_consent_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Approve access to your account" }, "children": [] },
            { "block": "button", "props": { "label": "Allow", "variant": "primary" }, "children": [] },
            { "block": "button", "props": { "label": "Deny", "variant": "outline" }, "children": [] }
        ]
    })
}

fn default_magic_link_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "We sent a magic link to your inbox." }, "children": [] },
            { "block": "button", "props": { "label": "Resend link", "variant": "secondary" }, "children": [] }
        ]
    })
}

fn default_error_blueprint() -> Value {
    json!({
        "layout": "default",
        "blocks": [
            { "block": "text", "props": { "text": "Something went wrong." }, "children": [] },
            { "block": "button", "props": { "label": "Try again", "variant": "primary" }, "children": [] }
        ]
    })
}
