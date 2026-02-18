use crate::application::flow_manager::FlowManager;
use crate::application::realm_service::UpdateRealmPayload;
use crate::bootstrap::seed::context::SeedContext;
use crate::domain::auth_flow::AuthFlow;
use crate::domain::flow::models::FlowDraft;
use crate::domain::realm::Realm;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

pub async fn ensure_default_flows(
    ctx: &SeedContext<'_>,
    realm: &mut Realm,
) -> anyhow::Result<()> {
    let browser_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "browser-login",
        "Browser Login",
        "browser",
    )
    .await?;

    let direct_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "direct-grant",
        "Direct Grant",
        "direct",
    )
    .await?;

    let registration_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "registration",
        "Registration",
        "registration",
    )
    .await?;

    let reset_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "reset-credentials",
        "Reset Credentials",
        "reset",
    )
    .await?;

    let mut needs_update = false;
    let mut update_payload = UpdateRealmPayload {
        name: None,
        access_token_ttl_secs: None,
        refresh_token_ttl_secs: None,
        browser_flow_id: None,
        registration_flow_id: None,
        direct_grant_flow_id: None,
        reset_credentials_flow_id: None,
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

    if needs_update {
        ctx.realm_service.update_realm(realm.id, update_payload).await?;
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
        ctx.flow_repo.create_flow(&flow, None).await?;
        new_id
    };

    let draft_exists = ctx.flow_store.get_draft_by_id(&flow_id).await?.is_some();
    let graph_json = FlowManager::generate_default_graph(type_);
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

    if !draft_exists {
        ctx.flow_store.create_draft(&draft_obj).await?;
    }

    let latest_version = ctx.flow_store.get_latest_version_number(&flow_id).await?;
    let has_valid_version = latest_version.unwrap_or(0) > 0;

    if !has_valid_version {
        match ctx.flow_manager.publish_flow(*realm_id, flow_id).await {
            Ok(_) => {
                ctx.flow_store.create_draft(&draft_obj).await?;
            }
            Err(e) => {
                tracing::error!("FAILURE - Could not publish {}: {:?}", alias, e);
            }
        }
    }

    Ok(flow_id)
}
