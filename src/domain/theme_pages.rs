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
        key: "reset_password",
        label: "Reset Password",
        description: "Set a new password.",
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
        "reset_password" => Some(default_reset_password_blueprint()),
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
        "nodes": [
            {
                "type": "Text",
                "size": { "width": "fill", "height": "hug" },
                "props": { "text": "Page content" }
            }
        ]
    })
}

fn default_login_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Welcome back" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Email or username", "name": "username", "input_type": "text" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Password", "name": "password", "input_type": "password" } },
            { "type": "Component", "component": "Link", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Forgot password?", "href": "/forgot-password", "target": "_self", "align": "right" } },
            { "type": "Component", "component": "Link", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Create account", "href": "/register", "target": "_self", "align": "left", "visible_if": "capabilities.registration_enabled" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Continue", "variant": "primary" } }
        ]
    })
}

fn default_register_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Create your account" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Email", "name": "email", "input_type": "email" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Password", "name": "password", "input_type": "password" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Sign up", "variant": "primary" } }
        ]
    })
}

fn default_forgot_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Reset your password" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Username or email", "name": "username", "input_type": "text" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Send reset link", "variant": "primary" } }
        ]
    })
}

fn default_reset_password_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Set a new password" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "New password", "name": "password", "input_type": "password" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Confirm password", "name": "password_confirm", "input_type": "password" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Update password", "variant": "primary" } }
        ]
    })
}

fn default_verify_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Email verification complete." } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Continue", "variant": "primary" } }
        ]
    })
}

fn default_mfa_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Enter your verification code" } },
            { "type": "Component", "component": "Input", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Code", "name": "otp" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Verify", "variant": "primary" } }
        ]
    })
}

fn default_consent_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Approve access to your account" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Allow", "variant": "primary", "intent": "allow" } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Deny", "variant": "outline", "intent": "deny" } }
        ]
    })
}

fn default_magic_link_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "We sent a magic link to your inbox." } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Resend link", "variant": "secondary" } }
        ]
    })
}

fn default_error_blueprint() -> Value {
    json!({
        "layout": "default",
        "nodes": [
            { "type": "Text", "size": { "width": "fill", "height": "hug" }, "props": { "text": "Something went wrong." } },
            { "type": "Component", "component": "Button", "size": { "width": "fill", "height": "hug" }, "props": { "label": "Try again", "variant": "primary" } }
        ]
    })
}
