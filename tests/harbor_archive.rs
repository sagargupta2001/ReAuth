use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reauth::application::harbor::{
    read_bundle_from_path, write_bundle_to_path, HarborAsset, HarborBundle, HarborExportType,
    HarborManifest, HarborResourceBundle,
};
use serde_json::json;
use tempfile::tempdir;
use uuid::Uuid;

fn build_bundle() -> HarborBundle {
    let asset_id = Uuid::new_v4().to_string();
    let asset_bytes = b"asset-bytes".to_vec();
    let asset = HarborAsset {
        id: Some(asset_id),
        filename: "logo.png".to_string(),
        mime_type: "image/png".to_string(),
        asset_type: Some("logo".to_string()),
        data_base64: STANDARD.encode(asset_bytes),
    };

    let resource = HarborResourceBundle {
        key: "theme".to_string(),
        data: json!({
            "tokens": {},
            "layout": {},
            "nodes": [{"node_key": "root", "blueprint": {}}]
        }),
        assets: vec![asset],
        meta: None,
    };

    HarborBundle {
        manifest: HarborManifest {
            version: "1.0".to_string(),
            schema_version: 1,
            exported_at: "2026-03-04T10:00:00Z".to_string(),
            source_realm: "acme".to_string(),
            export_type: HarborExportType::Theme,
            selection: None,
        },
        resources: vec![resource],
    }
}

#[test]
fn harbor_archive_round_trip_zip() {
    let bundle = build_bundle();
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("bundle.reauth");

    write_bundle_to_path(&bundle, &path).expect("write bundle");
    let loaded = read_bundle_from_path(&path).expect("read bundle");

    let original = serde_json::to_value(&bundle).expect("bundle json");
    let parsed = serde_json::to_value(&loaded).expect("loaded json");
    assert_eq!(original, parsed);
}

#[test]
fn harbor_archive_round_trip_tar_gz() {
    let bundle = build_bundle();
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("bundle.tar.gz");

    write_bundle_to_path(&bundle, &path).expect("write bundle");
    let loaded = read_bundle_from_path(&path).expect("read bundle");

    let original = serde_json::to_value(&bundle).expect("bundle json");
    let parsed = serde_json::to_value(&loaded).expect("loaded json");
    assert_eq!(original, parsed);
}
