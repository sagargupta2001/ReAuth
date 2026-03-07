mod support;

use reauth::application::harbor::{
    ConflictPolicy, HarborBundle, HarborExportType, HarborManifest, HarborResourceBundle,
    HarborScope,
};
use reauth::application::realm_service::CreateRealmPayload;
use reauth::domain::oidc::OidcClient;
use reauth::error::Error;
use serde_json::{json, Value};
use support::TestContext;
use uuid::Uuid;

#[tokio::test]
async fn harbor_import_rejects_invalid_bundle_schema() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "schema-check".to_string(),
        })
        .await
        .expect("create realm");

    let resource = HarborResourceBundle {
        key: "flow".to_string(),
        data: json!("not-an-object"),
        assets: Vec::new(),
        meta: None,
    };

    let bundle = HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::FullRealm,
            selection: Some(vec!["flow".to_string()]),
        },
        resources: vec![resource],
    };

    let err = ctx
        .app_state
        .harbor_service
        .import_bundle(
            realm.id,
            HarborScope::FullRealm,
            bundle,
            false,
            ConflictPolicy::Overwrite,
        )
        .await
        .expect_err("expected validation error");

    match err {
        Error::Validation(_) => {}
        other => panic!("expected validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn harbor_import_conflict_skip_leaves_existing_client() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "conflict-skip".to_string(),
        })
        .await
        .expect("create realm");

    let mut existing = OidcClient {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: "portal-app".to_string(),
        client_secret: Some("secret".to_string()),
        redirect_uris: "[]".to_string(),
        scopes: "[]".to_string(),
        web_origins: "[]".to_string(),
        managed_by_config: false,
    };
    ctx.app_state
        .oidc_service
        .register_client(&mut existing)
        .await
        .expect("register client");

    let resource = HarborResourceBundle {
        key: "client".to_string(),
        data: json!({
            "client_id": "portal-app",
            "client_secret": null,
            "redirect_uris": ["https://example.com/callback"],
            "scopes": ["openid"],
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
        client_id: "portal-app".to_string(),
    };

    let result = ctx
        .app_state
        .harbor_service
        .import_bundle(realm.id, scope, bundle, false, ConflictPolicy::Skip)
        .await
        .expect("import bundle");

    assert_eq!(result.resources.len(), 1);
    assert_eq!(result.resources[0].status, "skipped");

    let persisted = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&realm.id, "portal-app")
        .await
        .expect("find client")
        .expect("client exists");

    assert_eq!(persisted.redirect_uris, "[]");
}

#[tokio::test]
async fn harbor_full_realm_remaps_client_ids_in_flows_and_bindings() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "remap".to_string(),
        })
        .await
        .expect("create realm");

    let mut existing = OidcClient {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: "portal-app".to_string(),
        client_secret: Some("secret".to_string()),
        redirect_uris: "[]".to_string(),
        scopes: "[]".to_string(),
        web_origins: "[]".to_string(),
        managed_by_config: false,
    };
    ctx.app_state
        .oidc_service
        .register_client(&mut existing)
        .await
        .expect("register client");

    let flow_id = Uuid::new_v4();
    let flow_graph = json!({
        "nodes": [
            {
                "id": "node-1",
                "type": "client",
                "config": {"client_id": "portal-app"}
            }
        ],
        "edges": []
    });

    let resources = vec![
        HarborResourceBundle {
            key: "client".to_string(),
            data: json!({
                "client_id": "portal-app",
                "client_secret": null,
                "redirect_uris": [],
                "scopes": [],
                "web_origins": []
            }),
            assets: Vec::new(),
            meta: None,
        },
        HarborResourceBundle {
            key: "flow".to_string(),
            data: json!({
                "name": "Browser Flow",
                "description": null,
                "flow_type": "browser",
                "graph_json": flow_graph,
                "flow_id": flow_id.to_string()
            }),
            assets: Vec::new(),
            meta: None,
        },
        HarborResourceBundle {
            key: "theme".to_string(),
            data: json!({
                "tokens": {},
                "layout": {},
                "nodes": [{"node_key": "login", "blueprint": {"nodes": []}}]
            }),
            assets: Vec::new(),
            meta: Some(json!({
                "draft_exists": true,
                "theme": {"name": "Remap Theme", "description": null, "is_system": false},
                "bindings": {"default": false, "clients": [{"client_id": "portal-app"}]}
            })),
        },
    ];

    let bundle = HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::FullRealm,
            selection: Some(vec![
                "client".to_string(),
                "flow".to_string(),
                "theme".to_string(),
            ]),
        },
        resources,
    };

    let result = ctx
        .app_state
        .harbor_service
        .import_bundle(
            realm.id,
            HarborScope::FullRealm,
            bundle,
            false,
            ConflictPolicy::Rename,
        )
        .await
        .expect("import bundle");

    assert!(result.resources.iter().any(|r| r.key == "client"));

    let renamed_client = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&realm.id, "portal-app-1")
        .await
        .expect("find renamed client");
    assert!(renamed_client.is_some());

    let draft = ctx
        .app_state
        .flow_manager
        .get_draft(flow_id)
        .await
        .expect("get flow draft");
    let graph: Value = serde_json::from_str(&draft.graph_json).expect("graph json");
    let mut client_ids = Vec::new();
    collect_client_ids(&graph, &mut client_ids);
    assert!(client_ids.contains(&"portal-app-1".to_string()));
    assert!(!client_ids.contains(&"portal-app".to_string()));

    let binding = ctx
        .app_state
        .theme_service
        .get_binding_for_client(realm.id, "portal-app-1")
        .await
        .expect("get binding");
    assert!(binding.is_some());

    let old_binding = ctx
        .app_state
        .theme_service
        .get_binding_for_client(realm.id, "portal-app")
        .await
        .expect("get old binding");
    assert!(old_binding.is_none());
}

#[tokio::test]
async fn harbor_import_allows_same_client_id_in_different_realms() {
    let ctx = TestContext::new_with_seed(false).await;
    let source_realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "harbor-source".to_string(),
        })
        .await
        .expect("create source realm");
    let target_realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "harbor-target".to_string(),
        })
        .await
        .expect("create target realm");

    let mut source_client = OidcClient {
        id: Uuid::new_v4(),
        realm_id: source_realm.id,
        client_id: "shared-client".to_string(),
        client_secret: Some("secret".to_string()),
        redirect_uris: "[\"https://example.com/callback\"]".to_string(),
        scopes: "[\"openid\"]".to_string(),
        web_origins: "[]".to_string(),
        managed_by_config: false,
    };
    ctx.app_state
        .oidc_service
        .register_client(&mut source_client)
        .await
        .expect("register source client");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source_realm.id,
            "harbor-source",
            HarborScope::Client {
                client_id: "shared-client".to_string(),
            },
            reauth::application::harbor::ExportPolicy::Redact,
            None,
        )
        .await
        .expect("export client bundle");

    ctx.app_state
        .harbor_service
        .import_bundle(
            target_realm.id,
            HarborScope::Client {
                client_id: "shared-client".to_string(),
            },
            bundle,
            false,
            ConflictPolicy::Overwrite,
        )
        .await
        .expect("import into target realm");

    let imported = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&target_realm.id, "shared-client")
        .await
        .expect("find target client");

    assert!(imported.is_some());
}

#[tokio::test]
async fn harbor_full_realm_theme_rename_creates_duplicate_theme() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "theme-rename".to_string(),
        })
        .await
        .expect("create realm");

    let existing = ctx
        .app_state
        .theme_service
        .create_theme(realm.id, "Portal".to_string(), Some("Existing".to_string()))
        .await
        .expect("create theme");

    ctx.app_state
        .theme_service
        .save_draft(
            realm.id,
            existing.id,
            serde_json::from_value(json!({
                "tokens": {},
                "layout": {},
                "nodes": [{"node_key": "login", "blueprint": {"nodes": []}}]
            }))
            .expect("draft"),
        )
        .await
        .expect("save draft");

    let bundle = HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::FullRealm,
            selection: Some(vec!["theme".to_string()]),
        },
        resources: vec![HarborResourceBundle {
            key: "theme".to_string(),
            data: json!({
                "tokens": {},
                "layout": {},
                "nodes": [{"node_key": "login", "blueprint": {"nodes": []}}]
            }),
            assets: Vec::new(),
            meta: Some(json!({
                "draft_exists": true,
                "theme": {"name": "Portal", "description": "Imported", "is_system": false},
                "bindings": {"default": false, "clients": []}
            })),
        }],
    };

    let result = ctx
        .app_state
        .harbor_service
        .import_bundle(
            realm.id,
            HarborScope::FullRealm,
            bundle,
            false,
            ConflictPolicy::Rename,
        )
        .await
        .expect("import bundle");

    assert_eq!(result.resources.len(), 1);
    assert_eq!(result.resources[0].key, "theme");

    let themes = ctx
        .app_state
        .theme_service
        .list_themes(realm.id)
        .await
        .expect("list themes");
    let theme_names = themes
        .into_iter()
        .map(|theme| theme.name)
        .collect::<Vec<_>>();

    assert!(theme_names.contains(&"Portal".to_string()));
    assert!(theme_names.contains(&"Portal-1".to_string()));
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.contains("renamed to 'Portal-1'")));
}

fn collect_client_ids(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(client_id)) = map.get("client_id") {
                out.push(client_id.clone());
            }
            for value in map.values() {
                collect_client_ids(value, out);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_client_ids(value, out);
            }
        }
        _ => {}
    }
}
