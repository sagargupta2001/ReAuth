#![allow(clippy::needless_option_as_deref)]

use crate::application::flow_manager::FlowManager;
use crate::application::realm_service::UpdateRealmPayload;
use crate::bootstrap::seed::context::SeedContext;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::flow::models::FlowDraft;
use crate::domain::realm::Realm;
use crate::ports::transaction_manager::Transaction;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

pub async fn ensure_default_flows(
    ctx: &SeedContext<'_>,
    realm: &mut Realm,
    tx: &mut Option<&mut dyn Transaction>,
) -> anyhow::Result<()> {
    let browser_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "browser-login",
        "Browser Login",
        "browser",
        tx,
    )
    .await?;

    let direct_flow_id =
        ensure_flow(ctx, &realm.id, "direct-grant", "Direct Grant", "direct", tx).await?;

    let registration_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "registration",
        "Registration",
        "registration",
        tx,
    )
    .await?;

    let reset_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "reset-credentials",
        "Reset Credentials",
        "reset",
        tx,
    )
    .await?;
    let invitation_flow_id =
        ensure_flow(ctx, &realm.id, "invitation", "Invitation", "invitation", tx).await?;

    let mut needs_update = false;
    let mut update_payload = UpdateRealmPayload {
        name: None,
        access_token_ttl_secs: None,
        refresh_token_ttl_secs: None,
        pkce_required_public_clients: None,
        lockout_threshold: None,
        lockout_duration_secs: None,
        registration_enabled: None,
        default_registration_role_ids: None,
        invitation_resend_limit: None,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
        invitation_flow_id: None,
    };

    if realm.browser_flow_id.is_none() {
        update_payload.browser_flow_id = Some(Some(browser_flow_id));
        needs_update = true;
    }
    if realm.direct_grant_flow_id.is_none() {
        update_payload.direct_grant_flow_id = Some(Some(direct_flow_id));
        needs_update = true;
    }
    if realm.registration_flow_id.is_none() {
        update_payload.registration_flow_id = Some(Some(registration_flow_id));
        needs_update = true;
    }
    if realm.reset_credentials_flow_id.is_none() {
        update_payload.reset_credentials_flow_id = Some(Some(reset_flow_id));
        needs_update = true;
    }
    if realm.invitation_flow_id.is_none() {
        update_payload.invitation_flow_id = Some(Some(invitation_flow_id));
        needs_update = true;
    }

    if needs_update {
        let tx_ref = tx.as_deref_mut();
        ctx.realm_service
            .update_realm_with_tx(realm.id, update_payload, tx_ref)
            .await?;
        info!("Updated realm with default flow bindings.");
        if let Some(updated) = ctx.realm_service.find_by_id(realm.id).await? {
            *realm = updated;
        }
    }

    Ok(())
}

async fn ensure_flow(
    ctx: &SeedContext<'_>,
    realm_id: &Uuid,
    name: &str,
    alias: &str,
    type_: &str,
    tx: &mut Option<&mut dyn Transaction>,
) -> anyhow::Result<Uuid> {
    let flow_id = if let Some(flow) = ctx.flow_repo.find_flow_by_name(realm_id, name).await? {
        flow.id
    } else {
        let new_id = Uuid::new_v4();
        let flow = AuthFlow {
            id: new_id,
            realm_id: *realm_id,
            name: name.to_string(),
            alias: alias.to_string(),
            description: Some(format!("Default {} flow", alias)),
            r#type: type_.to_string(),
            built_in: true,
        };
        let tx_ref = tx.as_deref_mut();
        ctx.flow_repo.create_flow(&flow, tx_ref).await?;
        new_id
    };

    let existing_draft = ctx.flow_store.get_draft_by_id(&flow_id).await?;
    let draft_exists = existing_draft.is_some();
    let graph_json = FlowManager::generate_default_graph(type_);
    let default_has_start = graph_contains_node_type(&graph_json, "core.start");
    let draft_missing_start = existing_draft
        .as_ref()
        .is_some_and(|draft| !graph_contains_node_type(&draft.graph_json, "core.start"));
    let default_has_recovery_issue =
        graph_contains_node_type(&graph_json, "core.logic.recovery_issue");
    let draft_missing_recovery_issue = existing_draft.as_ref().is_some_and(|draft| {
        !graph_contains_node_type(&draft.graph_json, "core.logic.recovery_issue")
    });
    let default_has_invitation_issue =
        graph_contains_node_type(&graph_json, "core.logic.issue_invitation");
    let draft_missing_invitation_issue = existing_draft.as_ref().is_some_and(|draft| {
        !graph_contains_node_type(&draft.graph_json, "core.logic.issue_invitation")
    });
    let default_has_invitation_start_edge = graph_has_edge_between_types(
        &graph_json,
        "core.start",
        None,
        "core.logic.invitation_token",
    );
    let draft_missing_invitation_start_edge = existing_draft.as_ref().is_some_and(|draft| {
        !graph_has_edge_between_types(
            &draft.graph_json,
            "core.start",
            None,
            "core.logic.invitation_token",
        )
    });
    let default_has_invitation_issue_edge = graph_has_edge_between_types(
        &graph_json,
        "core.logic.invitation_token",
        Some("valid"),
        "core.logic.issue_invitation",
    );
    let draft_missing_invitation_issue_edge = existing_draft.as_ref().is_some_and(|draft| {
        !graph_has_edge_between_types(
            &draft.graph_json,
            "core.logic.invitation_token",
            Some("valid"),
            "core.logic.issue_invitation",
        )
    });
    let default_has_invitation_validate_logic_type = graph_node_has_config_value(
        &graph_json,
        "core.logic.invitation_token",
        "logic_type",
        "core.logic.invitation_token",
    );
    let draft_missing_invitation_validate_logic_type =
        existing_draft.as_ref().is_some_and(|draft| {
            !graph_node_has_config_value(
                &draft.graph_json,
                "core.logic.invitation_token",
                "logic_type",
                "core.logic.invitation_token",
            )
        });
    let default_has_invitation_issue_logic_type = graph_node_has_config_value(
        &graph_json,
        "core.logic.issue_invitation",
        "logic_type",
        "core.logic.issue_invitation",
    );
    let draft_missing_invitation_issue_logic_type = existing_draft.as_ref().is_some_and(|draft| {
        !graph_node_has_config_value(
            &draft.graph_json,
            "core.logic.issue_invitation",
            "logic_type",
            "core.logic.issue_invitation",
        )
    });
    let default_has_invited_registration_override = graph_node_has_config_bool(
        &graph_json,
        "core.auth.register",
        "allow_when_invited",
        true,
    );
    let draft_missing_invited_registration_override =
        existing_draft.as_ref().is_some_and(|draft| {
            !graph_node_has_config_bool(
                &draft.graph_json,
                "core.auth.register",
                "allow_when_invited",
                true,
            )
        });
    let draft_obj = FlowDraft {
        id: flow_id,
        realm_id: *realm_id,
        name: alias.to_string(),
        description: Some(format!("Visual draft for {}", alias)),
        graph_json: graph_json.clone(),
        flow_type: type_.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut draft_updated = false;

    if !draft_exists {
        let tx_ref = tx.as_deref_mut();
        ctx.flow_store
            .create_draft_with_tx(&draft_obj, tx_ref)
            .await?;
    } else if (default_has_start && draft_missing_start)
        || (default_has_recovery_issue && draft_missing_recovery_issue)
        || (default_has_invitation_issue && draft_missing_invitation_issue)
        || (default_has_invitation_start_edge && draft_missing_invitation_start_edge)
        || (default_has_invitation_issue_edge && draft_missing_invitation_issue_edge)
        || (default_has_invitation_validate_logic_type
            && draft_missing_invitation_validate_logic_type)
        || (default_has_invitation_issue_logic_type && draft_missing_invitation_issue_logic_type)
        || (default_has_invited_registration_override
            && draft_missing_invited_registration_override)
    {
        let tx_ref = tx.as_deref_mut();
        ctx.flow_store
            .update_draft_with_tx(&draft_obj, tx_ref)
            .await?;
        draft_updated = true;
    }

    let latest_version = ctx.flow_store.get_latest_version_number(&flow_id).await?;
    let has_valid_version = latest_version.unwrap_or(0) > 0;

    if !has_valid_version || draft_updated {
        let tx_ref = tx.as_deref_mut();
        match ctx
            .flow_manager
            .publish_flow_with_tx(*realm_id, flow_id, tx_ref)
            .await
        {
            Ok(_) => {
                let tx_ref = tx.as_deref_mut();
                ctx.flow_store
                    .create_draft_with_tx(&draft_obj, tx_ref)
                    .await?;
            }
            Err(e) => {
                tracing::error!("FAILURE - Could not publish {}: {:?}", alias, e);
            }
        }
    }

    Ok(flow_id)
}

fn graph_contains_node_type(graph_json: &str, node_type: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(graph_json) else {
        return false;
    };
    value
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .map(|nodes| {
            nodes.iter().any(|node| {
                node.get("type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| value == node_type)
            })
        })
        .unwrap_or(false)
}

fn graph_node_has_config_value(
    graph_json: &str,
    node_type: &str,
    key: &str,
    expected: &str,
) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(graph_json) else {
        return false;
    };
    value
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .map(|nodes| {
            nodes.iter().any(|node| {
                node.get("type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| value == node_type)
                    && node
                        .get("data")
                        .and_then(|value| value.get("config"))
                        .and_then(|value| value.get(key))
                        .and_then(|value| value.as_str())
                        .is_some_and(|value| value == expected)
            })
        })
        .unwrap_or(false)
}

fn graph_node_has_config_bool(
    graph_json: &str,
    node_type: &str,
    key: &str,
    expected: bool,
) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(graph_json) else {
        return false;
    };
    value
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .map(|nodes| {
            nodes.iter().any(|node| {
                node.get("type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| value == node_type)
                    && node
                        .get("data")
                        .and_then(|value| value.get("config"))
                        .and_then(|value| value.get(key))
                        .and_then(|value| value.as_bool())
                        .is_some_and(|value| value == expected)
            })
        })
        .unwrap_or(false)
}

fn graph_has_edge_between_types(
    graph_json: &str,
    source_type: &str,
    source_handle: Option<&str>,
    target_type: &str,
) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(graph_json) else {
        return false;
    };
    let Some(nodes) = value.get("nodes").and_then(|nodes| nodes.as_array()) else {
        return false;
    };
    let Some(edges) = value.get("edges").and_then(|edges| edges.as_array()) else {
        return false;
    };

    let type_by_id = nodes
        .iter()
        .filter_map(|node| {
            let id = node.get("id").and_then(|value| value.as_str())?;
            let type_ = node.get("type").and_then(|value| value.as_str())?;
            Some((id.to_string(), type_.to_string()))
        })
        .collect::<std::collections::HashMap<_, _>>();

    edges.iter().any(|edge| {
        let Some(source_id) = edge.get("source").and_then(|value| value.as_str()) else {
            return false;
        };
        let Some(target_id) = edge.get("target").and_then(|value| value.as_str()) else {
            return false;
        };
        let source_matches = type_by_id
            .get(source_id)
            .is_some_and(|type_| type_ == source_type);
        let target_matches = type_by_id
            .get(target_id)
            .is_some_and(|type_| type_ == target_type);
        if !source_matches || !target_matches {
            return false;
        }

        match source_handle {
            Some(expected_handle) => edge
                .get("sourceHandle")
                .and_then(|value| value.as_str())
                .is_some_and(|handle| handle == expected_handle),
            None => true,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        graph_contains_node_type, graph_has_edge_between_types, graph_node_has_config_bool,
        graph_node_has_config_value, FlowManager,
    };

    #[test]
    fn invitation_default_graph_contains_issue_node_and_edges() {
        let graph = FlowManager::generate_default_graph("invitation");
        assert!(graph_contains_node_type(
            &graph,
            "core.logic.issue_invitation"
        ));
        assert!(graph_has_edge_between_types(
            &graph,
            "core.start",
            None,
            "core.logic.invitation_token"
        ));
        assert!(graph_has_edge_between_types(
            &graph,
            "core.logic.invitation_token",
            Some("valid"),
            "core.logic.issue_invitation"
        ));
        assert!(graph_node_has_config_value(
            &graph,
            "core.logic.invitation_token",
            "logic_type",
            "core.logic.invitation_token"
        ));
        assert!(graph_node_has_config_value(
            &graph,
            "core.logic.issue_invitation",
            "logic_type",
            "core.logic.issue_invitation"
        ));
        assert!(graph_node_has_config_bool(
            &graph,
            "core.auth.register",
            "allow_when_invited",
            true
        ));
    }
}
