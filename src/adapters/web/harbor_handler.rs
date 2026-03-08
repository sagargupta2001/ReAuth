use axum::body::Body;
use axum::extract::{Extension, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use axum::{response::Response, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::harbor::{
    bootstrap_import_bundle, read_bundle_from_path, resolve_bootstrap_realm_name,
    write_bundle_to_path, ConflictPolicy, ExportPolicy, HarborBundle, HarborImportResult,
    HarborScope,
};
use crate::domain::harbor_job::HarborJob;
use crate::domain::permissions;
use crate::domain::realm::Realm;
use crate::error::{Error, Result};
use crate::AppState;
use chrono::Utc;
use std::path::PathBuf;
use tracing::error;

#[derive(Deserialize)]
pub struct HarborExportRequest {
    pub scope: String,
    pub id: Option<String>,
    pub include_secrets: Option<bool>,
    pub selection: Option<Vec<String>>,
    pub archive_format: Option<String>,
}

#[derive(Deserialize)]
pub struct HarborImportRequest {
    pub scope: String,
    pub id: Option<String>,
    pub bundle: HarborBundle,
    #[serde(default)]
    pub conflict_policy: ConflictPolicy,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize)]
pub struct HarborBootstrapImportRequest {
    pub realm_name: Option<String>,
    pub bundle: HarborBundle,
    #[serde(default = "bootstrap_conflict_policy_default")]
    pub conflict_policy: ConflictPolicy,
}

#[derive(Deserialize)]
pub struct HarborImportQuery {
    pub dry_run: Option<bool>,
    #[serde(rename = "async")]
    pub async_mode: Option<bool>,
}

#[derive(Deserialize)]
pub struct HarborJobsQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct HarborExportQuery {
    #[serde(rename = "async")]
    pub async_mode: Option<bool>,
}

#[derive(serde::Serialize)]
pub struct HarborJobDetail {
    pub job: HarborJob,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

#[derive(serde::Serialize)]
pub struct HarborJobDetails {
    pub job: HarborJob,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    pub conflicts: Vec<crate::domain::harbor_job_conflict::HarborJobConflict>,
}

#[derive(Deserialize, Serialize)]
pub struct HarborAsyncResponse {
    pub job_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

#[derive(Serialize)]
pub struct HarborBootstrapImportResponse {
    pub realm: Realm,
    pub import: HarborImportResult,
}

#[derive(Serialize)]
pub struct HarborBootstrapAsyncResponse {
    pub realm: Realm,
    pub job_id: String,
}

pub async fn export_harbor_bundle_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(realm_name): Path<String>,
    Query(query): Query<HarborExportQuery>,
    Json(payload): Json<HarborExportRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let include_secrets = payload.include_secrets.unwrap_or(false);
    if include_secrets {
        let allowed = state
            .rbac_service
            .user_has_permission(&user_id, permissions::REALM_BACKUP)
            .await
            .unwrap_or(false);
        if !allowed {
            return Err(Error::SecurityViolation(
                "Include secrets requires full backup permission".to_string(),
            ));
        }
    }

    let scope = parse_scope(&payload)?;
    let policy = if include_secrets {
        ExportPolicy::IncludeSecrets
    } else {
        ExportPolicy::Redact
    };

    if query.async_mode.unwrap_or(false) {
        let format = payload
            .archive_format
            .clone()
            .unwrap_or_else(|| "zip".to_string());
        let response = schedule_async_export(
            &state,
            &realm,
            scope,
            policy,
            payload.selection.clone(),
            format,
        )
        .await?;
        return Ok((StatusCode::ACCEPTED, Json(response)).into_response());
    }

    let bundle = state
        .harbor_service
        .export_bundle(
            realm.id,
            &realm.name,
            scope,
            policy,
            payload.selection.clone(),
        )
        .await?;

    Ok((StatusCode::OK, Json(bundle)).into_response())
}

pub async fn export_harbor_archive_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(realm_name): Path<String>,
    Query(query): Query<HarborExportQuery>,
    Json(payload): Json<HarborExportRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name.clone()))?;

    let include_secrets = payload.include_secrets.unwrap_or(false);
    if include_secrets {
        let allowed = state
            .rbac_service
            .user_has_permission(&user_id, permissions::REALM_BACKUP)
            .await
            .unwrap_or(false);
        if !allowed {
            return Err(Error::SecurityViolation(
                "Include secrets requires full backup permission".to_string(),
            ));
        }
    }

    let scope = parse_scope(&payload)?;
    let policy = if include_secrets {
        ExportPolicy::IncludeSecrets
    } else {
        ExportPolicy::Redact
    };

    let selection = payload.selection.clone();
    let async_override = query.async_mode;
    let should_async = if !matches!(scope, HarborScope::FullRealm) {
        async_override.unwrap_or(false)
    } else {
        let threshold = state
            .settings
            .read()
            .await
            .harbor
            .async_export_threshold_resources;
        let estimated = state
            .harbor_service
            .estimate_export_size(realm.id, &scope, selection.clone())
            .await?;
        match async_override {
            Some(value) => value,
            None => estimated as usize >= threshold,
        }
    };

    let format = payload
        .archive_format
        .clone()
        .unwrap_or_else(|| "zip".to_string());
    if should_async {
        let response =
            schedule_async_export(&state, &realm, scope, policy, selection, format).await?;
        return Ok((StatusCode::ACCEPTED, Json(response)).into_response());
    }

    let bundle = state
        .harbor_service
        .export_bundle(realm.id, &realm.name, scope, policy, selection)
        .await?;

    let (tmp_path, filename, content_type) = build_archive_path(&realm.name, &format)?;

    write_bundle_to_path(&bundle, &tmp_path)?;
    let bytes = std::fs::read(&tmp_path).map_err(|e| Error::Unexpected(e.into()))?;
    let _ = std::fs::remove_file(&tmp_path);

    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .map_err(|_| Error::Validation("Invalid content type".to_string()))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| Error::Validation("Invalid archive filename".to_string()))?,
    );
    Ok(response.into_response())
}

pub async fn import_harbor_bundle_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<HarborImportQuery>,
    Json(payload): Json<HarborImportRequest>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let dry_run = query.dry_run.or(payload.dry_run).unwrap_or(false);
    let conflict_policy = payload.conflict_policy;
    let bundle = payload.bundle;
    let scope = parse_scope(&HarborExportRequest {
        scope: payload.scope.clone(),
        id: payload.id.clone(),
        include_secrets: None,
        selection: None,
        archive_format: None,
    })?;
    let async_override = query.async_mode;
    let threshold = state
        .settings
        .read()
        .await
        .harbor
        .async_import_threshold_resources;
    let async_job = if dry_run {
        false
    } else {
        match async_override {
            Some(value) => value,
            None => matches!(scope, HarborScope::FullRealm) && bundle.resources.len() >= threshold,
        }
    };

    if async_job {
        let total_resources = bundle.resources.len() as i64;
        let job_id = state
            .harbor_service
            .create_job(
                realm.id,
                "import",
                &scope,
                total_resources,
                dry_run,
                Some(conflict_policy),
            )
            .await?;

        let harbor = state.harbor_service.clone();
        let runner = state.harbor_service.clone();
        runner.spawn_job(Box::pin(async move {
            if let Err(err) = harbor
                .import_bundle_with_job(realm.id, scope, bundle, dry_run, conflict_policy, job_id)
                .await
            {
                error!("Harbor async import failed: {}", err);
            }
        }));

        return Ok((
            StatusCode::ACCEPTED,
            Json(HarborAsyncResponse {
                job_id: job_id.to_string(),
                download_url: None,
            }),
        )
            .into_response());
    }

    let result: HarborImportResult = state
        .harbor_service
        .import_bundle(realm.id, scope, bundle, dry_run, conflict_policy)
        .await?;

    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn bootstrap_import_harbor_bundle_handler(
    State(state): State<AppState>,
    Query(query): Query<HarborImportQuery>,
    Json(payload): Json<HarborBootstrapImportRequest>,
) -> Result<impl IntoResponse> {
    let bundle = payload.bundle;
    state
        .harbor_service
        .validate_bundle_for_scope(&bundle, &HarborScope::FullRealm)?;

    let dry_run = query.dry_run.unwrap_or(false);
    let realm_name = resolve_bootstrap_realm_name(payload.realm_name, &bundle)?;
    if dry_run {
        ensure_bootstrap_target_available(&state, &realm_name).await?;
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "dry_run": true,
                "realm_name": realm_name,
                "validated": true,
            })),
        )
            .into_response());
    }

    let async_override = query.async_mode;
    let threshold = state
        .settings
        .read()
        .await
        .harbor
        .async_import_threshold_resources;
    let async_job = match async_override {
        Some(value) => value,
        None => bundle.resources.len() >= threshold,
    };

    if async_job {
        let realm = create_bootstrap_realm(&state, &realm_name).await?;
        let total_resources = bundle.resources.len() as i64;
        let job_id = state
            .harbor_service
            .create_job(
                realm.id,
                "import",
                &HarborScope::FullRealm,
                total_resources,
                false,
                Some(payload.conflict_policy),
            )
            .await?;

        let harbor = state.harbor_service.clone();
        let runner = state.harbor_service.clone();
        runner.spawn_job(Box::pin(async move {
            if let Err(err) = harbor
                .import_bundle_with_job(
                    realm.id,
                    HarborScope::FullRealm,
                    bundle,
                    false,
                    payload.conflict_policy,
                    job_id,
                )
                .await
            {
                error!("Harbor bootstrap async import failed: {}", err);
            }
        }));

        return Ok((
            StatusCode::ACCEPTED,
            Json(HarborBootstrapAsyncResponse {
                realm,
                job_id: job_id.to_string(),
            }),
        )
            .into_response());
    }

    let (realm, import) = bootstrap_import_bundle(
        &state.realm_service,
        &state.harbor_service,
        Some(realm_name),
        bundle,
        payload.conflict_policy,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(HarborBootstrapImportResponse { realm, import }),
    )
        .into_response())
}

pub async fn list_harbor_jobs_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<HarborJobsQuery>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let jobs = state.harbor_service.list_jobs(realm.id, limit).await?;

    Ok((StatusCode::OK, Json(jobs)))
}

pub async fn list_harbor_job_conflicts_handler(
    State(state): State<AppState>,
    Path((realm_name, job_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let job = state.harbor_service.get_job(job_id).await?;
    let job = job.filter(|job| job.realm_id == realm.id);

    let Some(_job) = job else {
        return Err(Error::NotFound("Harbor job not found".to_string()));
    };

    let conflicts = state.harbor_service.list_job_conflicts(job_id).await?;

    Ok((StatusCode::OK, Json(conflicts)))
}

pub async fn get_harbor_job_handler(
    State(state): State<AppState>,
    Path((realm_name, job_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let job = state.harbor_service.get_job(job_id).await?;
    let job = job.filter(|job| job.realm_id == realm.id);

    let Some(job) = job else {
        return Err(Error::NotFound("Harbor job not found".to_string()));
    };

    let download_url = if job
        .artifact_path
        .as_ref()
        .map(|path| std::fs::metadata(path).is_ok())
        .unwrap_or(false)
    {
        Some(format!(
            "/api/realms/{}/harbor/jobs/{}/download",
            realm.name, job_id
        ))
    } else {
        None
    };

    Ok((StatusCode::OK, Json(HarborJobDetail { job, download_url })))
}

pub async fn get_harbor_job_details_handler(
    State(state): State<AppState>,
    Path((realm_name, job_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let job = state.harbor_service.get_job(job_id).await?;
    let job = job.filter(|job| job.realm_id == realm.id);

    let Some(job) = job else {
        return Err(Error::NotFound("Harbor job not found".to_string()));
    };

    let download_url = if job
        .artifact_path
        .as_ref()
        .map(|path| std::fs::metadata(path).is_ok())
        .unwrap_or(false)
    {
        Some(format!(
            "/api/realms/{}/harbor/jobs/{}/download",
            realm.name, job_id
        ))
    } else {
        None
    };

    let conflicts = state.harbor_service.list_job_conflicts(job_id).await?;

    Ok((
        StatusCode::OK,
        Json(HarborJobDetails {
            job,
            download_url,
            conflicts,
        }),
    ))
}

pub async fn download_harbor_job_handler(
    State(state): State<AppState>,
    Path((realm_name, job_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let job = state.harbor_service.get_job(job_id).await?;
    let job = job.filter(|job| job.realm_id == realm.id);

    let Some(job) = job else {
        return Err(Error::NotFound("Harbor job not found".to_string()));
    };

    if job.status != "completed" {
        return Err(Error::Validation("Harbor job is not completed".to_string()));
    }

    let path = job
        .artifact_path
        .ok_or_else(|| Error::NotFound("Harbor job artifact not available".to_string()))?;
    let filename = job
        .artifact_filename
        .unwrap_or_else(|| "harbor-export.reauth".to_string());
    let content_type = job
        .artifact_content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::NotFound(
                "Harbor artifact file not found (expired)".to_string(),
            ))
        }
        Err(err) => return Err(Error::Unexpected(err.into())),
    };

    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .map_err(|_| Error::Validation("Invalid content type".to_string()))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|_| Error::Validation("Invalid archive filename".to_string()))?,
    );

    Ok(response.into_response())
}

async fn schedule_async_export(
    state: &AppState,
    realm: &Realm,
    scope: HarborScope,
    policy: ExportPolicy,
    selection: Option<Vec<String>>,
    format: String,
) -> Result<HarborAsyncResponse> {
    let realm_id = realm.id;
    let total_resources = state
        .harbor_service
        .estimate_export_size(realm_id, &scope, selection.clone())
        .await?;
    let job_id = state
        .harbor_service
        .create_job(realm_id, "export", &scope, total_resources, false, None)
        .await?;

    let storage_dir = state.settings.read().await.harbor.storage_dir.clone();
    let realm_name = realm.name.clone();
    let harbor = state.harbor_service.clone();
    let runner = state.harbor_service.clone();
    let download_url = format!("/api/realms/{}/harbor/jobs/{}/download", realm_name, job_id);

    runner.spawn_job(Box::pin(async move {
        let result = harbor
            .export_bundle_with_job(
                realm_id,
                &realm_name,
                scope,
                policy,
                selection,
                job_id,
                false,
            )
            .await;

        match result {
            Ok(bundle) => {
                if let Err(err) = std::fs::create_dir_all(&storage_dir) {
                    let _ = harbor
                        .mark_job_failed(job_id, &format!("Failed to create storage dir: {}", err))
                        .await;
                    return;
                }

                match build_storage_archive_path(&storage_dir, &realm_name, &format) {
                    Ok((path, filename, content_type)) => {
                        if let Err(err) = write_bundle_to_path(&bundle, &path) {
                            let _ = harbor.mark_job_failed(job_id, &err.to_string()).await;
                            return;
                        }

                        if let Err(err) = harbor
                            .set_job_artifact(
                                job_id,
                                &path.to_string_lossy(),
                                &filename,
                                &content_type,
                            )
                            .await
                        {
                            let _ = harbor.mark_job_failed(job_id, &err.to_string()).await;
                            return;
                        }

                        let processed = bundle.resources.len() as i64;
                        let _ = harbor.mark_job_completed(job_id, processed, 0, 0).await;
                    }
                    Err(err) => {
                        let _ = harbor.mark_job_failed(job_id, &err.to_string()).await;
                    }
                }
            }
            Err(err) => {
                let _ = harbor.mark_job_failed(job_id, &err.to_string()).await;
            }
        }
    }));

    Ok(HarborAsyncResponse {
        job_id: job_id.to_string(),
        download_url: Some(download_url),
    })
}

pub async fn import_harbor_archive_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<HarborImportQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let mut scope: Option<String> = None;
    let mut id: Option<String> = None;
    let mut conflict_policy: Option<ConflictPolicy> = None;
    let mut dry_run: Option<bool> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::Validation(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "bundle" || name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| Error::Validation(e.to_string()))?;
            file_bytes = Some(bytes.to_vec());
            continue;
        }

        let text = field
            .text()
            .await
            .map_err(|e| Error::Validation(e.to_string()))?;
        match name.as_str() {
            "scope" => scope = Some(text),
            "id" => id = Some(text),
            "conflict_policy" => conflict_policy = Some(parse_conflict_policy(&text)?),
            "dry_run" => dry_run = Some(parse_bool(&text)?),
            _ => {}
        }
    }

    let Some(bytes) = file_bytes else {
        return Err(Error::Validation(
            "Multipart bundle file is required".to_string(),
        ));
    };

    let tmp_path = build_import_temp_path(file_name.as_deref())?;
    std::fs::write(&tmp_path, &bytes).map_err(|e| Error::Unexpected(e.into()))?;

    let bundle = read_bundle_from_path(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    let scope = parse_scope_fields(scope.as_deref(), id)?;
    let dry_run = query.dry_run.or(dry_run).unwrap_or(false);
    let conflict_policy = conflict_policy.unwrap_or_default();

    let async_override = query.async_mode;
    let threshold = state
        .settings
        .read()
        .await
        .harbor
        .async_import_threshold_resources;
    let async_job = if dry_run {
        false
    } else {
        match async_override {
            Some(value) => value,
            None => matches!(scope, HarborScope::FullRealm) && bundle.resources.len() >= threshold,
        }
    };

    if async_job {
        let total_resources = bundle.resources.len() as i64;
        let job_id = state
            .harbor_service
            .create_job(
                realm.id,
                "import",
                &scope,
                total_resources,
                dry_run,
                Some(conflict_policy),
            )
            .await?;

        let harbor = state.harbor_service.clone();
        let runner = state.harbor_service.clone();
        runner.spawn_job(Box::pin(async move {
            if let Err(err) = harbor
                .import_bundle_with_job(realm.id, scope, bundle, dry_run, conflict_policy, job_id)
                .await
            {
                error!("Harbor async import failed: {}", err);
            }
        }));

        return Ok((
            StatusCode::ACCEPTED,
            Json(HarborAsyncResponse {
                job_id: job_id.to_string(),
                download_url: None,
            }),
        )
            .into_response());
    }

    let result: HarborImportResult = state
        .harbor_service
        .import_bundle(realm.id, scope, bundle, dry_run, conflict_policy)
        .await?;

    Ok((StatusCode::OK, Json(result)).into_response())
}

pub async fn bootstrap_import_harbor_archive_handler(
    State(state): State<AppState>,
    Query(query): Query<HarborImportQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let mut realm_name: Option<String> = None;
    let mut conflict_policy: Option<ConflictPolicy> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::Validation(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "bundle" || name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| Error::Validation(e.to_string()))?;
            file_bytes = Some(bytes.to_vec());
            continue;
        }

        let text = field
            .text()
            .await
            .map_err(|e| Error::Validation(e.to_string()))?;
        match name.as_str() {
            "realm_name" => realm_name = Some(text),
            "conflict_policy" => conflict_policy = Some(parse_conflict_policy(&text)?),
            _ => {}
        }
    }

    let Some(bytes) = file_bytes else {
        return Err(Error::Validation(
            "Multipart bundle file is required".to_string(),
        ));
    };

    let tmp_path = build_import_temp_path(file_name.as_deref())?;
    std::fs::write(&tmp_path, &bytes).map_err(|e| Error::Unexpected(e.into()))?;

    let bundle = read_bundle_from_path(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    state
        .harbor_service
        .validate_bundle_for_scope(&bundle, &HarborScope::FullRealm)?;

    let dry_run = query.dry_run.unwrap_or(false);
    let realm_name = resolve_bootstrap_realm_name(realm_name, &bundle)?;
    let conflict_policy = conflict_policy.unwrap_or_else(bootstrap_conflict_policy_default);

    if dry_run {
        ensure_bootstrap_target_available(&state, &realm_name).await?;
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "dry_run": true,
                "realm_name": realm_name,
                "validated": true,
            })),
        )
            .into_response());
    }

    let async_override = query.async_mode;
    let threshold = state
        .settings
        .read()
        .await
        .harbor
        .async_import_threshold_resources;
    let async_job = match async_override {
        Some(value) => value,
        None => bundle.resources.len() >= threshold,
    };

    if async_job {
        let realm = create_bootstrap_realm(&state, &realm_name).await?;
        let total_resources = bundle.resources.len() as i64;
        let job_id = state
            .harbor_service
            .create_job(
                realm.id,
                "import",
                &HarborScope::FullRealm,
                total_resources,
                false,
                Some(conflict_policy),
            )
            .await?;

        let harbor = state.harbor_service.clone();
        let runner = state.harbor_service.clone();
        runner.spawn_job(Box::pin(async move {
            if let Err(err) = harbor
                .import_bundle_with_job(
                    realm.id,
                    HarborScope::FullRealm,
                    bundle,
                    false,
                    conflict_policy,
                    job_id,
                )
                .await
            {
                error!("Harbor bootstrap async import failed: {}", err);
            }
        }));

        return Ok((
            StatusCode::ACCEPTED,
            Json(HarborBootstrapAsyncResponse {
                realm,
                job_id: job_id.to_string(),
            }),
        )
            .into_response());
    }

    let (realm, import) = bootstrap_import_bundle(
        &state.realm_service,
        &state.harbor_service,
        Some(realm_name),
        bundle,
        conflict_policy,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(HarborBootstrapImportResponse { realm, import }),
    )
        .into_response())
}

fn parse_scope(payload: &HarborExportRequest) -> Result<HarborScope> {
    parse_scope_fields(Some(payload.scope.as_str()), payload.id.clone())
}

fn bootstrap_conflict_policy_default() -> ConflictPolicy {
    ConflictPolicy::Overwrite
}

async fn ensure_bootstrap_target_available(state: &AppState, realm_name: &str) -> Result<()> {
    if state
        .realm_service
        .find_by_name(realm_name)
        .await?
        .is_some()
    {
        return Err(Error::RealmAlreadyExists);
    }
    Ok(())
}

async fn create_bootstrap_realm(state: &AppState, realm_name: &str) -> Result<Realm> {
    ensure_bootstrap_target_available(state, realm_name).await?;
    state
        .realm_service
        .create_realm(crate::application::realm_service::CreateRealmPayload {
            name: realm_name.to_string(),
        })
        .await
}

fn parse_scope_fields(scope: Option<&str>, id: Option<String>) -> Result<HarborScope> {
    let scope = scope
        .map(|value| value.trim().to_lowercase())
        .ok_or_else(|| Error::Validation("Scope is required".to_string()))?;
    match scope.as_str() {
        "theme" => {
            let id = id.ok_or_else(|| Error::Validation("Theme scope requires id".to_string()))?;
            let theme_id = Uuid::parse_str(&id)
                .map_err(|_| Error::Validation("Invalid theme id".to_string()))?;
            Ok(HarborScope::Theme { theme_id })
        }
        "client" => {
            let id = id.ok_or_else(|| Error::Validation("Client scope requires id".to_string()))?;
            let client_id = id.trim().to_string();
            if client_id.is_empty() {
                return Err(Error::Validation("Client id is required".to_string()));
            }
            Ok(HarborScope::Client { client_id })
        }
        "flow" => {
            let id = id.ok_or_else(|| Error::Validation("Flow scope requires id".to_string()))?;
            let flow_id = Uuid::parse_str(&id)
                .map_err(|_| Error::Validation("Invalid flow id".to_string()))?;
            Ok(HarborScope::Flow { flow_id })
        }
        "user" => {
            let id = id.ok_or_else(|| Error::Validation("User scope requires id".to_string()))?;
            let user_id = Uuid::parse_str(&id)
                .map_err(|_| Error::Validation("Invalid user id".to_string()))?;
            Ok(HarborScope::User { user_id })
        }
        "role" => {
            let id = id.ok_or_else(|| Error::Validation("Role scope requires id".to_string()))?;
            let role_id = Uuid::parse_str(&id)
                .map_err(|_| Error::Validation("Invalid role id".to_string()))?;
            Ok(HarborScope::Role { role_id })
        }
        "full_realm" => Ok(HarborScope::FullRealm),
        _ => Err(Error::Validation("Unsupported harbor scope".to_string())),
    }
}

fn parse_conflict_policy(value: &str) -> Result<ConflictPolicy> {
    match value.trim().to_lowercase().as_str() {
        "skip" => Ok(ConflictPolicy::Skip),
        "overwrite" => Ok(ConflictPolicy::Overwrite),
        "rename" => Ok(ConflictPolicy::Rename),
        _ => Err(Error::Validation("Invalid conflict_policy".to_string())),
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(Error::Validation("Invalid boolean value".to_string())),
    }
}

fn build_archive_path(realm_name: &str, format: &str) -> Result<(PathBuf, String, String)> {
    let (suffix, content_type) = match format.trim().to_lowercase().as_str() {
        "zip" | "reauth" => ("reauth", "application/reauth+zip"),
        "tar" => ("tar", "application/x-tar"),
        "tar.gz" | "tgz" => ("tar.gz", "application/gzip"),
        other => {
            return Err(Error::Validation(format!(
                "Unsupported archive format: {}",
                other
            )))
        }
    };
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}-harbor-{}.{}", realm_name, timestamp, suffix);
    let mut path = std::env::temp_dir();
    path.push(format!("harbor-{}-{}", Uuid::new_v4(), filename));
    Ok((path, filename, content_type.to_string()))
}

fn build_storage_archive_path(
    storage_dir: &str,
    realm_name: &str,
    format: &str,
) -> Result<(PathBuf, String, String)> {
    let (suffix, content_type) = match format.trim().to_lowercase().as_str() {
        "zip" | "reauth" => ("reauth", "application/reauth+zip"),
        "tar" => ("tar", "application/x-tar"),
        "tar.gz" | "tgz" => ("tar.gz", "application/gzip"),
        other => {
            return Err(Error::Validation(format!(
                "Unsupported archive format: {}",
                other
            )))
        }
    };
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}-harbor-{}.{}", realm_name, timestamp, suffix);
    let mut path = PathBuf::from(storage_dir);
    path.push(format!("{}-{}", Uuid::new_v4(), filename));
    Ok((path, filename, content_type.to_string()))
}

fn build_import_temp_path(filename: Option<&str>) -> Result<PathBuf> {
    let ext = filename
        .and_then(|name| name.split('.').next_back())
        .map(|value| value.to_lowercase())
        .unwrap_or_else(|| "reauth".to_string());

    let mut path = std::env::temp_dir();
    if filename
        .map(|name| name.to_lowercase().ends_with(".tar.gz"))
        .unwrap_or(false)
    {
        path.push(format!("harbor-{}.tar.gz", Uuid::new_v4()));
        return Ok(path);
    }

    if filename
        .map(|name| name.to_lowercase().ends_with(".tgz"))
        .unwrap_or(false)
    {
        path.push(format!("harbor-{}.tgz", Uuid::new_v4()));
        return Ok(path);
    }

    if filename
        .map(|name| name.to_lowercase().ends_with(".tar"))
        .unwrap_or(false)
    {
        path.push(format!("harbor-{}.tar", Uuid::new_v4()));
        return Ok(path);
    }

    path.push(format!("harbor-{}.{}", Uuid::new_v4(), ext));
    Ok(path)
}
