use crate::adapters::web::auth_middleware::AuthUser;
use crate::domain::pagination::PageRequest;
use crate::domain::permissions;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchUserResult {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct SearchClientResult {
    pub id: Uuid,
    pub client_id: String,
}

#[derive(Debug, Serialize)]
pub struct SearchRoleResult {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub users: Vec<SearchUserResult>,
    pub clients: Vec<SearchClientResult>,
    pub roles: Vec<SearchRoleResult>,
}

impl SearchResponse {
    fn empty() -> Self {
        Self {
            users: vec![],
            clients: vec![],
            roles: vec![],
        }
    }
}

pub async fn omni_search_handler(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(query): Query<SearchQuery>,
    Extension(AuthUser(user)): Extension<AuthUser>,
) -> Result<impl IntoResponse> {
    let realm = state
        .realm_service
        .find_by_name(&realm_name)
        .await?
        .ok_or(Error::RealmNotFound(realm_name))?;

    let q = query.q.trim();
    if q.is_empty() {
        return Ok((StatusCode::OK, Json(SearchResponse::empty())));
    }

    let limit = query.limit.unwrap_or(6).clamp(1, 20);
    let page_req = PageRequest {
        page: 1,
        per_page: limit,
        sort_by: None,
        sort_dir: None,
        q: Some(q.to_string()),
    };

    let can_read_users = state
        .rbac_service
        .user_has_permission(&user.id, permissions::USER_READ)
        .await
        .unwrap_or(false);
    let can_read_clients = state
        .rbac_service
        .user_has_permission(&user.id, permissions::CLIENT_READ)
        .await
        .unwrap_or(false);
    let can_read_roles = state
        .rbac_service
        .user_has_permission(&user.id, permissions::RBAC_READ)
        .await
        .unwrap_or(false);

    let users = if can_read_users {
        let response = state
            .user_service
            .list_users(realm.id, page_req.clone())
            .await?;
        response
            .data
            .into_iter()
            .map(|user| SearchUserResult {
                id: user.id,
                username: user.username,
            })
            .collect()
    } else {
        vec![]
    };

    let clients = if can_read_clients {
        let response = state
            .oidc_service
            .list_clients(realm.id, page_req.clone())
            .await?;
        response
            .data
            .into_iter()
            .map(|client| SearchClientResult {
                id: client.id,
                client_id: client.client_id,
            })
            .collect()
    } else {
        vec![]
    };

    let roles = if can_read_roles {
        let response = state.rbac_service.list_roles(realm.id, page_req).await?;
        response
            .data
            .into_iter()
            .map(|role| SearchRoleResult {
                id: role.id,
                name: role.name,
                description: role.description,
                client_id: role.client_id,
            })
            .collect()
    } else {
        vec![]
    };

    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            users,
            clients,
            roles,
        }),
    ))
}
