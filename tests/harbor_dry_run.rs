mod support;

use reauth::application::harbor::{
    ConflictPolicy, HarborBundle, HarborExportType, HarborManifest, HarborResourceBundle,
    HarborScope,
};
use reauth::application::realm_service::CreateRealmPayload;
use serde_json::json;
use support::TestContext;

#[tokio::test]
async fn harbor_import_dry_run_does_not_persist() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "dry-run".to_string(),
        })
        .await
        .expect("create realm");

    let resource = HarborResourceBundle {
        key: "client".to_string(),
        data: json!({
            "client_id": "dry-client",
            "client_secret": null,
            "redirect_uris": [],
            "scopes": [],
            "web_origins": []
        }),
        assets: Vec::new(),
        meta: None,
    };

    let bundle = HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::Client,
            selection: None,
        },
        resources: vec![resource],
    };

    let scope = HarborScope::Client {
        client_id: "dry-client".to_string(),
    };

    let result = ctx
        .app_state
        .harbor_service
        .import_bundle(realm.id, scope, bundle, true, ConflictPolicy::Overwrite)
        .await
        .expect("dry run import");

    assert!(result.dry_run);
    assert_eq!(result.resources.len(), 1);
    assert_eq!(result.resources[0].created, 1);

    let existing = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&realm.id, "dry-client")
        .await
        .expect("find client");

    assert!(existing.is_none());
}
