use std::sync::Arc;

use uuid::Uuid;

use crate::domain::user_phone_number::{normalize_phone_number, UserPhoneNumber};
use crate::error::{Error, Result};
use crate::ports::transaction_manager::TransactionManager;
use crate::ports::user_phone_number_repository::UserPhoneNumberRepository;

pub struct UserPhoneNumberService {
    phone_number_repo: Arc<dyn UserPhoneNumberRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl UserPhoneNumberService {
    pub fn new(
        phone_number_repo: Arc<dyn UserPhoneNumberRepository>,
        tx_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            phone_number_repo,
            tx_manager,
        }
    }

    pub async fn list_phone_numbers(&self, user_id: Uuid) -> Result<Vec<UserPhoneNumber>> {
        self.phone_number_repo.find_by_user_id(&user_id).await
    }

    pub async fn get_primary_phone_number(&self, user_id: Uuid) -> Result<Option<UserPhoneNumber>> {
        self.phone_number_repo.find_primary(&user_id).await
    }

    pub async fn add_phone_number(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        phone_number: &str,
        is_primary: bool,
        is_verified: bool,
    ) -> Result<UserPhoneNumber> {
        let display_value = phone_number.trim();
        let normalized = normalize_phone_number(display_value);
        if display_value.is_empty() || normalized.is_empty() {
            return Err(Error::Validation(
                "Phone number cannot be empty".to_string(),
            ));
        }

        if self
            .phone_number_repo
            .find_by_phone_number(&realm_id, &normalized)
            .await?
            .is_some()
        {
            return Err(Error::PhoneNumberAlreadyExists);
        }

        let record = UserPhoneNumber::new(
            user_id,
            realm_id,
            display_value.to_string(),
            is_primary,
            is_verified,
        );

        let mut tx = self.tx_manager.begin().await?;
        self.phone_number_repo.save(&record, Some(&mut *tx)).await?;
        self.tx_manager.commit(tx).await?;

        Ok(record)
    }

    pub async fn remove_phone_number(&self, user_id: Uuid, phone_number_id: Uuid) -> Result<()> {
        let all = self.phone_number_repo.find_by_user_id(&user_id).await?;
        let target = all
            .iter()
            .find(|phone_number| phone_number.id == phone_number_id)
            .ok_or_else(|| Error::NotFound("Phone number not found".to_string()))?;

        if target.is_primary && all.len() > 1 {
            return Err(Error::Conflict(
                "Cannot remove the primary phone number while other numbers exist. Set another phone number as primary first.".to_string(),
            ));
        }

        self.phone_number_repo.delete(&phone_number_id, None).await
    }

    pub async fn set_primary(&self, user_id: Uuid, phone_number_id: Uuid) -> Result<()> {
        let all = self.phone_number_repo.find_by_user_id(&user_id).await?;
        if !all
            .iter()
            .any(|phone_number| phone_number.id == phone_number_id)
        {
            return Err(Error::NotFound("Phone number not found".to_string()));
        }

        let mut tx = self.tx_manager.begin().await?;
        self.phone_number_repo
            .set_primary(&user_id, &phone_number_id, Some(&mut *tx))
            .await?;
        self.tx_manager.commit(tx).await?;
        Ok(())
    }

    pub async fn set_verified(
        &self,
        user_id: Uuid,
        phone_number_id: Uuid,
        is_verified: bool,
    ) -> Result<()> {
        let all = self.phone_number_repo.find_by_user_id(&user_id).await?;
        if !all
            .iter()
            .any(|phone_number| phone_number.id == phone_number_id)
        {
            return Err(Error::NotFound("Phone number not found".to_string()));
        }

        self.phone_number_repo
            .set_verified(&phone_number_id, is_verified, None)
            .await
    }
}
