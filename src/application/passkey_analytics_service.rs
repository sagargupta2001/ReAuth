use crate::domain::audit::AuditEvent;
use crate::error::{Error, Result};
use crate::ports::audit_repository::AuditRepository;
use crate::ports::passkey_challenge_repository::PasskeyChallengeRepository;
use crate::ports::passkey_credential_repository::PasskeyCredentialRepository;
use crate::ports::realm_repository::RealmRepository;
use chrono::{Duration, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

const PASSKEY_AUDIT_ACTIONS: &[&str] = &[
    "passkey.assertion.success",
    "passkey.assertion.invalid_signature",
    "passkey.assertion.counter_regression",
    "passkey.assertion.challenge_mismatch",
    "passkey.enrollment.success",
    "passkey.enrollment.challenge_mismatch",
];

const PASSKEY_FAILURE_ACTIONS: &[&str] = &[
    "passkey.assertion.invalid_signature",
    "passkey.assertion.counter_regression",
    "passkey.assertion.challenge_mismatch",
    "passkey.enrollment.challenge_mismatch",
];

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyOutcomeMetrics {
    pub assertion_success: u64,
    pub assertion_invalid_signature: u64,
    pub assertion_counter_regression: u64,
    pub assertion_challenge_mismatch: u64,
    pub enrollment_success: u64,
    pub enrollment_challenge_mismatch: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyChallengeMetrics {
    pub pending_total: u64,
    pub pending_expired: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyFailureEvent {
    pub action: String,
    pub created_at: String,
    pub actor_user_id: Option<Uuid>,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyAnalyticsSnapshot {
    pub realm_id: Uuid,
    pub window_hours: i64,
    pub credentials_total: u64,
    pub credentials_created_last_7d: u64,
    pub credentials_active_last_30d: u64,
    pub challenges: PasskeyChallengeMetrics,
    pub outcomes: PasskeyOutcomeMetrics,
    pub recent_failures: Vec<PasskeyFailureEvent>,
}

pub struct PasskeyAnalyticsService {
    realm_repo: Arc<dyn RealmRepository>,
    audit_repo: Arc<dyn AuditRepository>,
    credential_repo: Arc<dyn PasskeyCredentialRepository>,
    challenge_repo: Arc<dyn PasskeyChallengeRepository>,
}

impl PasskeyAnalyticsService {
    pub fn new(
        realm_repo: Arc<dyn RealmRepository>,
        audit_repo: Arc<dyn AuditRepository>,
        credential_repo: Arc<dyn PasskeyCredentialRepository>,
        challenge_repo: Arc<dyn PasskeyChallengeRepository>,
    ) -> Self {
        Self {
            realm_repo,
            audit_repo,
            credential_repo,
            challenge_repo,
        }
    }

    pub async fn snapshot(
        &self,
        realm_id: Uuid,
        window_hours: i64,
        recent_limit: usize,
    ) -> Result<PasskeyAnalyticsSnapshot> {
        if window_hours <= 0 || window_hours > 24 * 30 {
            return Err(Error::Validation(
                "window_hours must be between 1 and 720".to_string(),
            ));
        }
        if recent_limit == 0 || recent_limit > 100 {
            return Err(Error::Validation(
                "recent_limit must be between 1 and 100".to_string(),
            ));
        }

        let realm = self
            .realm_repo
            .find_by_id(&realm_id)
            .await?
            .ok_or_else(|| Error::RealmNotFound(realm_id.to_string()))?;

        let now = Utc::now();
        let window_start = now - Duration::hours(window_hours);
        let created_since = now - Duration::days(7);
        let active_since = now - Duration::days(30);

        let credentials_total = self.credential_repo.count_by_realm(&realm.id).await?;
        let credentials_created_last_7d = self
            .credential_repo
            .count_created_since(&realm.id, created_since)
            .await?;
        let credentials_active_last_30d = self
            .credential_repo
            .count_active_since(&realm.id, active_since)
            .await?;
        let pending_total = self.challenge_repo.count_unconsumed(&realm.id).await?;
        let pending_expired = self
            .challenge_repo
            .count_expired_unconsumed(&realm.id, now)
            .await?;

        let action_counts = self
            .audit_repo
            .count_by_actions_since(&realm.id, PASSKEY_AUDIT_ACTIONS, Some(window_start))
            .await?;
        let mut count_by_action: HashMap<&str, u64> = HashMap::new();
        for row in action_counts {
            if let Some(key) = PASSKEY_AUDIT_ACTIONS
                .iter()
                .find(|action| **action == row.action.as_str())
            {
                count_by_action.insert(*key, row.count);
            }
        }

        let recent_failures = self
            .audit_repo
            .list_recent_by_actions(&realm.id, PASSKEY_FAILURE_ACTIONS, recent_limit)
            .await?
            .into_iter()
            .map(map_failure_event)
            .collect();

        Ok(PasskeyAnalyticsSnapshot {
            realm_id: realm.id,
            window_hours,
            credentials_total,
            credentials_created_last_7d,
            credentials_active_last_30d,
            challenges: PasskeyChallengeMetrics {
                pending_total,
                pending_expired,
            },
            outcomes: PasskeyOutcomeMetrics {
                assertion_success: *count_by_action
                    .get("passkey.assertion.success")
                    .unwrap_or(&0),
                assertion_invalid_signature: *count_by_action
                    .get("passkey.assertion.invalid_signature")
                    .unwrap_or(&0),
                assertion_counter_regression: *count_by_action
                    .get("passkey.assertion.counter_regression")
                    .unwrap_or(&0),
                assertion_challenge_mismatch: *count_by_action
                    .get("passkey.assertion.challenge_mismatch")
                    .unwrap_or(&0),
                enrollment_success: *count_by_action
                    .get("passkey.enrollment.success")
                    .unwrap_or(&0),
                enrollment_challenge_mismatch: *count_by_action
                    .get("passkey.enrollment.challenge_mismatch")
                    .unwrap_or(&0),
            },
            recent_failures,
        })
    }
}

fn map_failure_event(event: AuditEvent) -> PasskeyFailureEvent {
    PasskeyFailureEvent {
        action: event.action,
        created_at: event.created_at,
        actor_user_id: event.actor_user_id,
        target_id: event.target_id,
    }
}
