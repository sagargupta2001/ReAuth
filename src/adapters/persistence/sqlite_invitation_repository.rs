use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::pagination::{PageRequest, PageResponse, SortDirection};
use crate::error::{Error, Result};
use crate::ports::invitation_repository::InvitationRepository;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Sqlite};
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteInvitationRepository {
    pool: Database,
}

impl SqliteInvitationRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn apply_filters<'a>(
        builder: &mut QueryBuilder<'a, Sqlite>,
        realm_id: &Uuid,
        q: &Option<String>,
        status: Option<InvitationStatus>,
    ) {
        builder.push(" WHERE realm_id = ");
        builder.push_bind(realm_id.to_string());

        if let Some(status) = status {
            builder.push(" AND status = ");
            builder.push_bind(status.to_string());
        }

        if let Some(query_text) = q {
            if !query_text.is_empty() {
                builder.push(" AND email LIKE ");
                builder.push_bind(format!("%{}%", query_text));
            }
        }
    }
}

#[async_trait]
impl InvitationRepository for SqliteInvitationRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "insert")
    )]
    async fn create(
        &self,
        invitation: &Invitation,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO invitations (
                id, realm_id, email, email_normalized, status, token_hash, expiry_days, expires_at,
                invited_by_user_id, accepted_user_id, accepted_at, revoked_at, resend_count,
                last_sent_at, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(invitation.id.to_string())
        .bind(invitation.realm_id.to_string())
        .bind(&invitation.email)
        .bind(&invitation.email_normalized)
        .bind(invitation.status.to_string())
        .bind(&invitation.token_hash)
        .bind(invitation.expiry_days)
        .bind(invitation.expires_at)
        .bind(invitation.invited_by_user_id.map(|id| id.to_string()))
        .bind(invitation.accepted_user_id.map(|id| id.to_string()))
        .bind(invitation.accepted_at)
        .bind(invitation.revoked_at)
        .bind(invitation.resend_count)
        .bind(invitation.last_sent_at)
        .bind(invitation.created_at)
        .bind(invitation.updated_at);

        if let Some(tx) = tx {
            let sqlite_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query.execute(&mut **sqlite_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "update")
    )]
    async fn update(
        &self,
        invitation: &Invitation,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE invitations
             SET status = ?, token_hash = ?, accepted_user_id = ?, accepted_at = ?, revoked_at = ?,
                 resend_count = ?, last_sent_at = ?, updated_at = ?
             WHERE id = ? AND realm_id = ?",
        )
        .bind(invitation.status.to_string())
        .bind(&invitation.token_hash)
        .bind(invitation.accepted_user_id.map(|id| id.to_string()))
        .bind(invitation.accepted_at)
        .bind(invitation.revoked_at)
        .bind(invitation.resend_count)
        .bind(invitation.last_sent_at)
        .bind(invitation.updated_at)
        .bind(invitation.id.to_string())
        .bind(invitation.realm_id.to_string());

        if let Some(tx) = tx {
            let sqlite_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query.execute(&mut **sqlite_tx).await
        } else {
            query.execute(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "select")
    )]
    async fn find_by_id(&self, realm_id: &Uuid, id: &Uuid) -> Result<Option<Invitation>> {
        let invitation = sqlx::query_as::<_, Invitation>(
            "SELECT * FROM invitations WHERE realm_id = ? AND id = ?",
        )
        .bind(realm_id.to_string())
        .bind(id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(invitation)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "select")
    )]
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Invitation>> {
        let invitation =
            sqlx::query_as::<_, Invitation>("SELECT * FROM invitations WHERE token_hash = ?")
                .bind(token_hash)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(invitation)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "select")
    )]
    async fn find_pending_by_email(
        &self,
        realm_id: &Uuid,
        email_normalized: &str,
    ) -> Result<Option<Invitation>> {
        let invitation = sqlx::query_as::<_, Invitation>(
            "SELECT * FROM invitations
             WHERE realm_id = ? AND email_normalized = ? AND status = 'pending'",
        )
        .bind(realm_id.to_string())
        .bind(email_normalized)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(invitation)
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "update")
    )]
    async fn expire_pending_before(&self, realm_id: &Uuid, cutoff: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE invitations
             SET status = 'expired', updated_at = ?
             WHERE realm_id = ? AND status = 'pending' AND expires_at <= ?",
        )
        .bind(Utc::now())
        .bind(realm_id.to_string())
        .bind(cutoff)
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(result.rows_affected())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "invitations", db_op = "select")
    )]
    async fn list(
        &self,
        realm_id: &Uuid,
        req: &PageRequest,
        status: Option<InvitationStatus>,
    ) -> Result<PageResponse<Invitation>> {
        let limit = req.per_page.clamp(1, 100);
        let offset = (req.page - 1) * limit;

        let mut count_builder = QueryBuilder::new("SELECT COUNT(*) FROM invitations");
        Self::apply_filters(&mut count_builder, realm_id, &req.q, status);
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        let mut query_builder = QueryBuilder::new("SELECT * FROM invitations");
        Self::apply_filters(&mut query_builder, realm_id, &req.q, status);

        let sort_col = match req.sort_by.as_deref() {
            Some("email") => "email",
            Some("status") => "status",
            Some("expires_at") => "expires_at",
            Some("last_sent_at") => "last_sent_at",
            Some("created_at") => "created_at",
            _ => "created_at",
        };
        let sort_dir = match req.sort_dir.unwrap_or(SortDirection::Desc) {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        query_builder.push(format!(" ORDER BY {} {}", sort_col, sort_dir));

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let invitations: Vec<Invitation> = query_builder
            .build_query_as()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(PageResponse::new(invitations, total, req.page, limit))
    }
}
