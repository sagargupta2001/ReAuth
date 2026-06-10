use std::sync::Arc;

use uuid::Uuid;

use crate::domain::user_email::UserEmail;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::TransactionManager;
use crate::ports::user_email_repository::UserEmailRepository;

pub struct UserEmailService {
    email_repo: Arc<dyn UserEmailRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl UserEmailService {
    pub fn new(
        email_repo: Arc<dyn UserEmailRepository>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            email_repo,
            tx_manager,
        }
    }

    pub async fn list_emails(&self, user_id: Uuid) -> Result<Vec<UserEmail>> {
        self.email_repo.find_by_user_id(&user_id).await
    }

    pub async fn get_primary_email(&self, user_id: Uuid) -> Result<Option<UserEmail>> {
        self.email_repo.find_primary(&user_id).await
    }

    /// Add a new email address to a user. Enforces realm-level uniqueness.
    pub async fn add_email(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        email: &str,
        is_primary: bool,
        is_verified: bool,
    ) -> Result<UserEmail> {
        let normalized = email.trim().to_lowercase();
        if normalized.is_empty() {
            return Err(Error::Validation("Email cannot be empty".to_string()));
        }

        if self
            .email_repo
            .find_by_email(&realm_id, &normalized)
            .await?
            .is_some()
        {
            return Err(Error::EmailAlreadyExists);
        }

        let record = UserEmail::new(
            user_id,
            realm_id,
            email.to_string(),
            is_primary,
            is_verified,
        );

        let mut tx = self.tx_manager.begin().await?;
        self.email_repo.save(&record, Some(&mut *tx)).await?;
        self.tx_manager.commit(tx).await?;

        Ok(record)
    }

    /// Remove an email address. Prevents removing the only/primary email without promotion.
    pub async fn remove_email(&self, user_id: Uuid, email_id: Uuid) -> Result<()> {
        let all = self.email_repo.find_by_user_id(&user_id).await?;
        let target = all
            .iter()
            .find(|e| e.id == email_id)
            .ok_or_else(|| Error::NotFound("Email address not found".to_string()))?;

        if target.is_primary && all.len() > 1 {
            return Err(Error::Conflict(
                "Cannot remove the primary email while other addresses exist. Set another email as primary first.".to_string(),
            ));
        }

        self.email_repo.delete(&email_id, None).await
    }

    /// Atomically set an email as primary (demotes the previous primary via DB trigger).
    pub async fn set_primary(&self, user_id: Uuid, email_id: Uuid) -> Result<()> {
        // Verify the email belongs to this user
        let all = self.email_repo.find_by_user_id(&user_id).await?;
        if !all.iter().any(|e| e.id == email_id) {
            return Err(Error::NotFound("Email address not found".to_string()));
        }

        let mut tx = self.tx_manager.begin().await?;
        self.email_repo
            .set_primary(&user_id, &email_id, Some(&mut *tx))
            .await?;
        self.tx_manager.commit(tx).await?;
        Ok(())
    }

    /// Update the verified flag on an email address.
    pub async fn set_verified(
        &self,
        user_id: Uuid,
        email_id: Uuid,
        is_verified: bool,
    ) -> Result<()> {
        let all = self.email_repo.find_by_user_id(&user_id).await?;
        if !all.iter().any(|e| e.id == email_id) {
            return Err(Error::NotFound("Email address not found".to_string()));
        }
        self.email_repo
            .set_verified(&email_id, is_verified, None)
            .await
    }
}
