use crate::application::harbor::{ConflictPolicy, HarborBundle, HarborImportResult, HarborScope};
use crate::application::realm_service::{CreateRealmPayload, RealmService};
use crate::domain::realm::Realm;
use crate::error::{Error, Result};

use super::service::HarborService;

pub fn resolve_bootstrap_realm_name(
    requested_realm_name: Option<String>,
    bundle: &HarborBundle,
) -> Result<String> {
    let candidate = requested_realm_name
        .unwrap_or_else(|| bundle.manifest.source_realm.clone())
        .trim()
        .to_string();

    if candidate.is_empty() {
        return Err(Error::Validation(
            "Bootstrap import requires a target realm name".to_string(),
        ));
    }

    Ok(candidate)
}

pub async fn bootstrap_import_bundle(
    realm_service: &RealmService,
    harbor_service: &HarborService,
    requested_realm_name: Option<String>,
    bundle: HarborBundle,
    conflict_policy: ConflictPolicy,
) -> Result<(Realm, HarborImportResult)> {
    harbor_service.validate_bundle_for_scope(&bundle, &HarborScope::FullRealm)?;

    let realm_name = resolve_bootstrap_realm_name(requested_realm_name, &bundle)?;
    let realm = realm_service
        .create_realm(CreateRealmPayload { name: realm_name })
        .await?;

    let result = harbor_service
        .import_bundle(
            realm.id,
            HarborScope::FullRealm,
            bundle,
            false,
            conflict_policy,
        )
        .await?;

    Ok((realm, result))
}
