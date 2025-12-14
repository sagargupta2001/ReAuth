use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents an in-progress login attempt.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginSession {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    #[sqlx(try_from = "String")]
    pub flow_id: Uuid,
    pub current_step: i64,
    pub user_id: Option<Uuid>,
    pub state_data: Option<String>,
    pub expires_at: DateTime<Utc>,
    #[sqlx(skip)] // todo Skip SQL mapping for now to prevent runtime errors if column is missing
    pub context: Value,
}

impl Default for LoginSession {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id: Uuid::default(),
            flow_id: Uuid::default(),
            current_step: 0,
            user_id: None,
            state_data: None,
            context: serde_json::json!({}), // Default to empty JSON object
            expires_at: Utc::now(),
        }
    }
}

/// Represents a configured flow (e.g., "browser-login")
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuthFlow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub alias: String,
    pub r#type: String, // "type" is a reserved keyword
    pub built_in: bool,
}

/// Represents a single step in a flow
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuthFlowStep {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub flow_id: Uuid,
    pub authenticator_name: String,
    pub priority: i64,
    pub requirement: String,    // 'REQUIRED', 'ALTERNATIVE', etc.
    pub config: Option<String>, // JSON
    pub parent_step_id: Option<String>,
}

/// Represents the configuration for a specific authenticator
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthenticatorConfig {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub authenticator_name: String,
    pub config_data: String,
}

/// Context passed to every authenticator step.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub realm_id: Uuid,
    pub login_session: LoginSession,
    pub config: Option<AuthenticatorConfig>,
    // HTTP form data, etc.
    pub credentials: HashMap<String, String>,
}

/// The result of an authenticator's `execute` or `challenge` method.
#[derive(Debug, Serialize)]
// 1. "tag" flattens the enum and puts the variant name in a "status" field
// 2. "rename_all" makes the status value lowercase ("challenge" instead of "Challenge")
#[serde(tag = "status", rename_all = "camelCase")]
pub enum AuthStepResult {
    /// The step (or entire flow) passed.
    Success,
    /// The step failed.
    Failure { message: String },
    /// The flow must pause and challenge the user for more info.
    Challenge {
        /// The type of challenge (e.g., "FORM", "OTP", "WEBAUTHN")
        challenge_name: String,
        /// The frontend route to render (e.g., "/login", "/mfa-otp")
        challenge_page: String,
    },
    /// The flow is complete, redirect the user to the OIDC callback.
    Redirect { url: String },
}
