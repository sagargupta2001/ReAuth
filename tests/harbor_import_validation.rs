mod support;

use reauth::application::flow_manager::{CreateDraftRequest, UpdateDraftRequest};
use reauth::application::harbor::{
    bootstrap_import_bundle, ConflictPolicy, ExportPolicy, HarborBundle, HarborExportType,
    HarborManifest, HarborResourceBundle, HarborScope,
};
use reauth::application::rbac_service::CreateRolePayload;
use reauth::application::realm_service::{CreateRealmPayload, UpdateRealmPayload};
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
    let _ = ctx
        .app_state
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
    let _ = ctx
        .app_state
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
    let _ = ctx
        .app_state
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

#[tokio::test]
async fn harbor_full_realm_imports_roles_and_remaps_client_roles() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "roles-remap".to_string(),
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
    let _ = ctx
        .app_state
        .oidc_service
        .register_client(&mut existing)
        .await
        .expect("register client");

    let bundle = HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::FullRealm,
            selection: Some(vec!["client".to_string(), "role".to_string()]),
        },
        resources: vec![
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
                key: "role".to_string(),
                data: json!({
                    "role_id": Uuid::new_v4().to_string(),
                    "name": "manage-portal",
                    "description": "Portal role",
                    "client_id": "portal-app",
                    "permissions": ["client:read", "client:update"]
                }),
                assets: Vec::new(),
                meta: None,
            },
        ],
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

    assert!(result
        .resources
        .iter()
        .any(|resource| resource.key == "role"));

    let renamed_client = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&realm.id, "portal-app-1")
        .await
        .expect("find renamed client")
        .expect("renamed client exists");

    let client_roles = ctx
        .app_state
        .rbac_service
        .list_client_roles(
            realm.id,
            renamed_client.id,
            reauth::domain::pagination::PageRequest::default(),
        )
        .await
        .expect("list client roles");

    let role = client_roles
        .data
        .into_iter()
        .find(|role| role.name == "manage-portal")
        .expect("client role exists");

    let permissions = ctx
        .app_state
        .rbac_service
        .get_role(realm.id, role.id)
        .await
        .expect("get role");
    assert_eq!(permissions.name, "manage-portal");
}

#[tokio::test]
async fn harbor_full_realm_exports_roles_when_selected() {
    let ctx = TestContext::new_with_seed(false).await;
    let realm = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "roles-export".to_string(),
        })
        .await
        .expect("create realm");

    ctx.app_state
        .rbac_service
        .create_role(
            realm.id,
            CreateRolePayload {
                name: "realm-admin".to_string(),
                description: Some("Admin role".to_string()),
                client_id: None,
            },
        )
        .await
        .expect("create role");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            realm.id,
            "roles-export",
            HarborScope::FullRealm,
            reauth::application::harbor::ExportPolicy::Redact,
            Some(vec!["role".to_string()]),
        )
        .await
        .expect("export bundle");

    assert_eq!(bundle.manifest.selection, Some(vec!["role".to_string()]));
    assert!(bundle
        .resources
        .iter()
        .any(|resource| resource.key == "role"));
}

#[tokio::test]
async fn harbor_bootstrap_import_creates_new_realm_from_full_bundle() {
    let ctx = TestContext::new_with_seed(false).await;
    let source = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "source-bootstrap".to_string(),
        })
        .await
        .expect("create source realm");

    let mut client = OidcClient {
        id: Uuid::new_v4(),
        realm_id: source.id,
        client_id: "portal-app".to_string(),
        client_secret: Some("secret".to_string()),
        redirect_uris: "[\"https://example.com/callback\"]".to_string(),
        scopes: "[\"openid\"]".to_string(),
        web_origins: "[]".to_string(),
        managed_by_config: false,
    };
    let _ = ctx
        .app_state
        .oidc_service
        .register_client(&mut client)
        .await
        .expect("register source client");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source.id,
            &source.name,
            HarborScope::FullRealm,
            ExportPolicy::Redact,
            Some(vec!["client".to_string()]),
        )
        .await
        .expect("export full realm bundle");

    let (target, result) = bootstrap_import_bundle(
        &ctx.app_state.realm_service,
        &ctx.app_state.harbor_service,
        Some("target-bootstrap".to_string()),
        bundle,
        ConflictPolicy::Overwrite,
    )
    .await
    .expect("bootstrap import");

    assert_eq!(target.name, "target-bootstrap");
    assert!(result
        .resources
        .iter()
        .any(|resource| resource.key == "client"));

    let imported = ctx
        .app_state
        .oidc_service
        .find_client_by_client_id(&target.id, "portal-app")
        .await
        .expect("find imported client");

    assert!(imported.is_some());
}

#[tokio::test]
async fn harbor_bootstrap_import_uses_manifest_source_realm_name_when_missing() {
    let ctx = TestContext::new_with_seed(false).await;
    let source = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "manifest-source".to_string(),
        })
        .await
        .expect("create source realm");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source.id,
            "manifest-bootstrap-target",
            HarborScope::FullRealm,
            ExportPolicy::Redact,
            Some(vec!["theme".to_string()]),
        )
        .await
        .expect("export full realm bundle");

    let (target, _) = bootstrap_import_bundle(
        &ctx.app_state.realm_service,
        &ctx.app_state.harbor_service,
        None,
        bundle,
        ConflictPolicy::Overwrite,
    )
    .await
    .expect("bootstrap import");

    assert_eq!(target.name, "manifest-bootstrap-target");
}

#[tokio::test]
async fn harbor_bootstrap_import_restores_realm_settings_and_flow_bindings() {
    let ctx = TestContext::new_with_seed(false).await;
    let source = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "realm-settings-source".to_string(),
        })
        .await
        .expect("create source realm");

    let browser_draft = ctx
        .app_state
        .flow_manager
        .create_draft(
            source.id,
            CreateDraftRequest {
                name: "custom-browser".to_string(),
                description: Some("Custom browser flow".to_string()),
                flow_type: "browser".to_string(),
            },
        )
        .await
        .expect("create draft");

    ctx.app_state
        .flow_manager
        .update_draft(
            browser_draft.id,
            UpdateDraftRequest {
                name: None,
                description: None,
                graph_json: Some(json!({
                    "nodes": [
                        {"id": "start", "type": "core.start"},
                        {"id": "end", "type": "core.terminal.allow"}
                    ],
                    "edges": [
                        {"source": "start", "target": "end"}
                    ]
                })),
            },
        )
        .await
        .expect("update draft graph");

    ctx.app_state
        .flow_manager
        .publish_flow(source.id, browser_draft.id)
        .await
        .expect("publish flow");

    ctx.app_state
        .realm_service
        .update_realm(
            source.id,
            UpdateRealmPayload {
                name: None,
                access_token_ttl_secs: Some(1800),
                refresh_token_ttl_secs: Some(14400),
                pkce_required_public_clients: Some(false),
                lockout_threshold: Some(7),
                lockout_duration_secs: Some(1200),
                registration_enabled: None,
                default_registration_role_ids: None,
                browser_flow_id: Some(Some(browser_draft.id)),
                registration_flow_id: Some(None),
                direct_grant_flow_id: Some(None),
                reset_credentials_flow_id: Some(None),
            },
        )
        .await
        .expect("update source realm");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source.id,
            &source.name,
            HarborScope::FullRealm,
            ExportPolicy::Redact,
            Some(vec!["realm".to_string(), "flow".to_string()]),
        )
        .await
        .expect("export realm bundle");

    let (target, _) = bootstrap_import_bundle(
        &ctx.app_state.realm_service,
        &ctx.app_state.harbor_service,
        Some("realm-settings-target".to_string()),
        bundle,
        ConflictPolicy::Overwrite,
    )
    .await
    .expect("bootstrap import");

    let imported_realm = ctx
        .app_state
        .realm_service
        .find_by_id(target.id)
        .await
        .expect("lookup target realm")
        .expect("target realm exists");

    assert_eq!(imported_realm.access_token_ttl_secs, 1800);
    assert_eq!(imported_realm.refresh_token_ttl_secs, 14400);
    assert!(!imported_realm.pkce_required_public_clients);
    assert_eq!(imported_realm.lockout_threshold, 7);
    assert_eq!(imported_realm.lockout_duration_secs, 1200);
    let imported_browser_flow_id = imported_realm
        .browser_flow_id
        .clone()
        .expect("browser flow binding restored");
    assert_ne!(
        imported_browser_flow_id,
        source.browser_flow_id.unwrap_or_default()
    );

    let imported_flows = ctx
        .app_state
        .flow_service
        .list_flows(target.id)
        .await
        .expect("list target flows");
    assert!(imported_flows
        .iter()
        .any(|flow| flow.id.to_string() == imported_browser_flow_id));
}

#[tokio::test]
async fn harbor_full_realm_bootstrap_imports_users_with_credentials_and_roles() {
    let ctx = TestContext::new_with_seed(false).await;
    let source = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "user-source".to_string(),
        })
        .await
        .expect("create source realm");

    let role = ctx
        .app_state
        .rbac_service
        .create_role(
            source.id,
            CreateRolePayload {
                name: "admin".to_string(),
                ..CreateRolePayload::default()
            },
        )
        .await
        .expect("create role");

    let user = ctx
        .app_state
        .user_service
        .create_user(source.id, "alice", "password-123", None, false)
        .await
        .expect("create user");

    ctx.app_state
        .rbac_service
        .assign_role_to_user(source.id, user.id, role.id)
        .await
        .expect("assign role");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source.id,
            &source.name,
            HarborScope::FullRealm,
            ExportPolicy::IncludeSecrets,
            Some(vec!["role".to_string(), "user".to_string()]),
        )
        .await
        .expect("export bundle");

    let (target, _) = bootstrap_import_bundle(
        &ctx.app_state.realm_service,
        &ctx.app_state.harbor_service,
        Some("user-target".to_string()),
        bundle,
        ConflictPolicy::Overwrite,
    )
    .await
    .expect("bootstrap import");

    let imported_user = ctx
        .app_state
        .user_service
        .find_by_username(&target.id, "alice")
        .await
        .expect("find user")
        .expect("user exists");
    assert_eq!(imported_user.hashed_password, user.hashed_password);

    let direct_role_ids = ctx
        .app_state
        .rbac_service
        .get_direct_role_ids_for_user(target.id, imported_user.id)
        .await
        .expect("list direct roles");
    assert_eq!(direct_role_ids.len(), 1);
}

#[tokio::test]
async fn harbor_bootstrap_rejects_new_user_creation_when_credentials_are_redacted() {
    let ctx = TestContext::new_with_seed(false).await;
    let source = ctx
        .app_state
        .realm_service
        .create_realm(CreateRealmPayload {
            name: "redacted-user-source".to_string(),
        })
        .await
        .expect("create source realm");

    ctx.app_state
        .user_service
        .create_user(source.id, "alice", "password-123", None, false)
        .await
        .expect("create user");

    let bundle = ctx
        .app_state
        .harbor_service
        .export_bundle(
            source.id,
            &source.name,
            HarborScope::FullRealm,
            ExportPolicy::Redact,
            Some(vec!["user".to_string()]),
        )
        .await
        .expect("export bundle");

    let err = bootstrap_import_bundle(
        &ctx.app_state.realm_service,
        &ctx.app_state.harbor_service,
        Some("redacted-user-target".to_string()),
        bundle,
        ConflictPolicy::Overwrite,
    )
    .await
    .expect_err("expected redacted credential import failure");

    match err {
        Error::Validation(message) => {
            assert!(message.contains("credentials are redacted"));
        }
        other => panic!("expected validation error, got: {:?}", other),
    }
}
