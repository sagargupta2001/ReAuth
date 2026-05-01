use super::service::HarborService;
use crate::application::harbor::types::*;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use uuid::Uuid;

use super::utils::*;
impl HarborService {
    pub async fn import_bundle(
        &self,
        realm_id: Uuid,
        scope: HarborScope,
        bundle: HarborBundle,
        dry_run: bool,
        conflict_policy: ConflictPolicy,
    ) -> Result<HarborImportResult> {
        self.import_bundle_internal(realm_id, scope, bundle, dry_run, conflict_policy, None)
            .await
    }

    pub async fn import_bundle_with_job(
        &self,
        realm_id: Uuid,
        scope: HarborScope,
        bundle: HarborBundle,
        dry_run: bool,
        conflict_policy: ConflictPolicy,
        job_id: Uuid,
    ) -> Result<HarborImportResult> {
        self.import_bundle_internal(
            realm_id,
            scope,
            bundle,
            dry_run,
            conflict_policy,
            Some(job_id),
        )
        .await
    }

    pub(crate) async fn import_bundle_internal(
        &self,
        realm_id: Uuid,
        scope: HarborScope,
        bundle: HarborBundle,
        dry_run: bool,
        conflict_policy: ConflictPolicy,
        job_override: Option<Uuid>,
    ) -> Result<HarborImportResult> {
        let bundle = upgrade_bundle(bundle)?;
        let job_id = match job_override {
            Some(id) => Some(id),
            None => {
                self.start_job(
                    realm_id,
                    super::service::HARBOR_JOB_TYPE_IMPORT,
                    &scope,
                    bundle.resources.len() as i64,
                    dry_run,
                    Some(conflict_policy),
                )
                .await
            }
        };

        if let Err(err) = self.validate_bundle(&bundle, &scope) {
            if let Some(job_id) = job_id {
                self.try_mark_failed(job_id, &err).await;
            }
            return Err(err);
        }

        let result = if dry_run {
            let mut tx = self.tx_manager.begin().await?;
            let result = self
                .import_bundle_with_tx(
                    realm_id,
                    scope,
                    bundle,
                    conflict_policy,
                    job_id,
                    Some(&mut *tx),
                )
                .await;
            self.tx_manager.rollback(tx).await?;
            let mut result = result?;
            result.dry_run = true;
            Ok(result)
        } else {
            self.import_bundle_with_tx(realm_id, scope, bundle, conflict_policy, job_id, None)
                .await
        };

        match result {
            Ok(result) => {
                if let Some(job_id) = job_id {
                    let (created, updated) = summarize_import_counts(&result);
                    let processed = result.resources.len() as i64;
                    self.try_mark_completed(job_id, processed, created, updated)
                        .await;
                }
                Ok(result)
            }
            Err(err) => {
                if let Some(job_id) = job_id {
                    self.try_mark_failed(job_id, &err).await;
                }
                Err(err)
            }
        }
    }

    pub(crate) async fn import_bundle_with_tx(
        &self,
        realm_id: Uuid,
        scope: HarborScope,
        bundle: HarborBundle,
        conflict_policy: ConflictPolicy,
        job_id: Option<Uuid>,
        mut tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResult> {
        let persist_job_updates = tx.is_none();
        if let HarborScope::FullRealm = scope {
            return self
                .import_full_bundle(
                    realm_id,
                    bundle,
                    conflict_policy,
                    job_id,
                    persist_job_updates,
                    tx.as_deref_mut(),
                )
                .await;
        }

        let provider_key = scope
            .provider_key()
            .ok_or_else(|| Error::Validation("Unsupported import scope".to_string()))?;
        let provider = self.registry.get(provider_key).ok_or_else(|| {
            Error::System(format!(
                "Harbor provider not registered for scope: {}",
                provider_key
            ))
        })?;

        let resource = bundle
            .resources
            .iter()
            .find(|resource| resource.key == provider_key)
            .ok_or_else(|| Error::Validation("Bundle missing required resource".to_string()))?;

        let result = provider
            .import(realm_id, &scope, resource, conflict_policy, false, tx)
            .await?;

        if persist_job_updates {
            if let Some(job_id) = job_id {
                self.try_update_job_progress(
                    job_id,
                    1,
                    result.created as i64,
                    result.updated as i64,
                )
                .await;
                self.try_record_conflict(job_id, &result, conflict_policy)
                    .await;
            }
        }

        Ok(HarborImportResult {
            dry_run: false,
            resources: vec![result],
            warnings: Vec::new(),
        })
    }
}
