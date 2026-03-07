use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HarborJobConflict {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub job_id: Uuid,
    pub resource_key: String,
    pub action: String,
    pub policy: String,
    pub original_id: Option<String>,
    pub resolved_id: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}
