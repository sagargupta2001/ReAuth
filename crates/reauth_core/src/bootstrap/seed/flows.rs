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

    let direct_flow_id = ensure_flow(
        ctx,
        &realm.id,
        "direct-grant",
        "Direct Grant",
        "direct",
        tx,
    )
    .await?;

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
        let tx_ref = tx.as_mut().map(|inner| &mut **inner);
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
        let tx_ref = tx.as_mut().map(|inner| &mut **inner);
        ctx.flow_repo.create_flow(&flow, tx_ref).await?;
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
        let tx_ref = tx.as_mut().map(|inner| &mut **inner);
        ctx.flow_store
            .create_draft_with_tx(&draft_obj, tx_ref)
            .await?;
    }

    let latest_version = ctx.flow_store.get_latest_version_number(&flow_id).await?;
    let has_valid_version = latest_version.unwrap_or(0) > 0;

    if !has_valid_version {
        let tx_ref = tx.as_mut().map(|inner| &mut **inner);
        match ctx
            .flow_manager
            .publish_flow_with_tx(*realm_id, flow_id, tx_ref)
            .await
        {
            Ok(_) => {
                let tx_ref = tx.as_mut().map(|inner| &mut **inner);
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
