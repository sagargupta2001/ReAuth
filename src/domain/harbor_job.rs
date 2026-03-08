use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HarborJob {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub realm_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub scope: String,
    pub total_resources: i64,
    pub processed_resources: i64,
    pub created_count: i64,
    pub updated_count: i64,
    pub dry_run: bool,
    pub conflict_policy: Option<String>,
    pub artifact_path: Option<String>,
    pub artifact_filename: Option<String>,
    pub artifact_content_type: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
