use crate::adapters::web::auth_middleware::AuthUser;
use crate::domain::pagination::PageRequest;
use crate::domain::permissions;
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct SearchGroupResult {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchFlowResult {
    pub id: Uuid,
    pub alias: String,
    pub description: Option<String>,
    pub flow_type: String,
    pub built_in: bool,
    pub is_draft: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchWebhookResult {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub users: Vec<SearchUserResult>,
    pub clients: Vec<SearchClientResult>,
    pub roles: Vec<SearchRoleResult>,
    pub groups: Vec<SearchGroupResult>,
    pub flows: Vec<SearchFlowResult>,
    pub webhooks: Vec<SearchWebhookResult>,
}

impl SearchResponse {
    fn empty() -> Self {
        Self {
            users: vec![],
            clients: vec![],
            roles: vec![],
            groups: vec![],
            flows: vec![],
            webhooks: vec![],
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
    let can_read_groups = can_read_roles;
    let can_read_flows = state
        .rbac_service
        .user_has_permission(&user.id, permissions::REALM_READ)
        .await
        .unwrap_or(false);
    let can_read_webhooks = can_read_flows;

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
        let response = state
            .rbac_service
            .list_roles(realm.id, page_req.clone())
            .await?;
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

    let groups = if can_read_groups {
        let response = state
            .rbac_service
            .list_groups(realm.id, page_req.clone())
            .await?;
        response
            .data
            .into_iter()
            .map(|group| SearchGroupResult {
                id: group.id,
                name: group.name,
                description: group.description,
            })
            .collect()
    } else {
        vec![]
    };

    let flows = if can_read_flows {
        let q_lower = q.to_lowercase();
        let runtime_flows = state.flow_service.list_flows(realm.id).await?;
        let drafts = state.flow_manager.list_all_drafts(realm.id).await?;
        let mut flows_map: HashMap<Uuid, SearchFlowResult> = HashMap::new();

        for flow in runtime_flows {
            flows_map.insert(
                flow.id,
                SearchFlowResult {
                    id: flow.id,
                    alias: flow.alias,
                    description: flow.description,
                    flow_type: flow.r#type,
                    built_in: flow.built_in,
                    is_draft: false,
                },
            );
        }

        for draft in drafts {
            flows_map
                .entry(draft.id)
                .and_modify(|existing| {
                    existing.alias = draft.name.clone();
                    existing.description = draft.description.clone();
                    existing.is_draft = true;
                    existing.flow_type = draft.flow_type.clone();
                })
                .or_insert_with(|| SearchFlowResult {
                    id: draft.id,
                    alias: draft.name,
                    description: draft.description,
                    flow_type: draft.flow_type,
                    built_in: false,
                    is_draft: true,
                });
        }

        let mut flows: Vec<SearchFlowResult> = flows_map
            .into_values()
            .filter(|flow| {
                flow.alias.to_lowercase().contains(&q_lower)
                    || flow
                        .description
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&q_lower)
                    || flow.flow_type.to_lowercase().contains(&q_lower)
            })
            .collect();

        flows.sort_by(|a, b| match (a.built_in, b.built_in) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.alias.cmp(&b.alias),
        });

        flows.truncate(limit as usize);
        flows
    } else {
        vec![]
    };

    let webhooks = if can_read_webhooks {
        let results = state
            .webhook_service
            .search_endpoints(realm.id, q, limit)
            .await?;
        results
            .into_iter()
            .map(|endpoint| SearchWebhookResult {
                id: endpoint.id,
                name: endpoint.name,
                url: endpoint.url,
                http_method: endpoint.http_method,
                status: endpoint.status,
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
            groups,
            flows,
            webhooks,
        }),
    ))
}
