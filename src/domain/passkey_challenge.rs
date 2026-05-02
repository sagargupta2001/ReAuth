use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasskeyChallengeKind {
    Authentication,
    Enrollment,
    Reauthentication,
}

impl PasskeyChallengeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "authentication",
            Self::Enrollment => "enrollment",
            Self::Reauthentication => "reauthentication",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyChallenge {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub auth_session_id: Uuid,
    pub user_id: Option<Uuid>,
    pub challenge_kind: PasskeyChallengeKind,
    pub challenge_hash: String,
    pub rp_id: String,
    pub allowed_origins_json: String,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl PasskeyChallenge {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_id: Uuid,
        auth_session_id: Uuid,
        user_id: Option<Uuid>,
        challenge_kind: PasskeyChallengeKind,
        challenge_hash: String,
        rp_id: String,
        allowed_origins_json: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id,
            auth_session_id,
            user_id,
            challenge_kind,
            challenge_hash,
            rp_id,
            allowed_origins_json,
            expires_at,
            consumed_at: None,
            created_at: Utc::now(),
        }
    }
}
