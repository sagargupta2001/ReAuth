use super::{FlowDeployment, FlowDraft, FlowVersion, NodeMetadata};
use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

#[test]
fn flow_draft_round_trip() {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let draft = FlowDraft {
        id: Uuid::new_v4(),
        realm_id: Uuid::new_v4(),
        name: "Draft".to_string(),
        description: Some("desc".to_string()),
        graph_json: "[]".to_string(),
        flow_type: "browser".to_string(),
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_string(&draft).expect("serialize");
    let decoded: FlowDraft = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, draft.id);
    assert_eq!(decoded.realm_id, draft.realm_id);
    assert_eq!(decoded.name, draft.name);
    assert_eq!(decoded.description, draft.description);
    assert_eq!(decoded.graph_json, draft.graph_json);
    assert_eq!(decoded.flow_type, draft.flow_type);
    assert_eq!(decoded.created_at, draft.created_at);
    assert_eq!(decoded.updated_at, draft.updated_at);
}

#[test]
fn flow_version_round_trip() {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let version = FlowVersion {
        id: "v1".to_string(),
        flow_id: "flow".to_string(),
        version_number: 1,
        execution_artifact: "artifact".to_string(),
        graph_json: "{}".to_string(),
        checksum: "checksum".to_string(),
        created_at: now,
    };

    let json = serde_json::to_string(&version).expect("serialize");
    let decoded: FlowVersion = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, version.id);
    assert_eq!(decoded.flow_id, version.flow_id);
    assert_eq!(decoded.version_number, version.version_number);
    assert_eq!(decoded.execution_artifact, version.execution_artifact);
    assert_eq!(decoded.graph_json, version.graph_json);
    assert_eq!(decoded.checksum, version.checksum);
    assert_eq!(decoded.created_at, version.created_at);
}

#[test]
fn flow_deployment_round_trip() {
    let now = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
    let deployment = FlowDeployment {
        id: "deployment".to_string(),
        realm_id: Uuid::new_v4(),
        flow_type: "browser".to_string(),
        active_version_id: "v1".to_string(),
        updated_at: now,
    };

    let json = serde_json::to_string(&deployment).expect("serialize");
    let decoded: FlowDeployment = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, deployment.id);
    assert_eq!(decoded.realm_id, deployment.realm_id);
    assert_eq!(decoded.flow_type, deployment.flow_type);
    assert_eq!(decoded.active_version_id, deployment.active_version_id);
    assert_eq!(decoded.updated_at, deployment.updated_at);
}

#[test]
fn node_metadata_round_trip() {
    let metadata = NodeMetadata {
        id: "node".to_string(),
        category: "Logic".to_string(),
        display_name: "Node".to_string(),
        description: "desc".to_string(),
        icon: "icon".to_string(),
        config_schema: json!({"type": "object"}),
        inputs: vec!["in".to_string()],
        outputs: vec!["out".to_string()],
    };

    let json = serde_json::to_string(&metadata).expect("serialize");
    let decoded: NodeMetadata = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.id, metadata.id);
    assert_eq!(decoded.category, metadata.category);
    assert_eq!(decoded.display_name, metadata.display_name);
    assert_eq!(decoded.description, metadata.description);
    assert_eq!(decoded.icon, metadata.icon);
    assert_eq!(decoded.config_schema, metadata.config_schema);
    assert_eq!(decoded.inputs, metadata.inputs);
    assert_eq!(decoded.outputs, metadata.outputs);
}

#[tokio::test]
async fn flow_models_from_row_parse_uuid_fields() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");

    let now = Utc::now();
    let draft_id = Uuid::new_v4();
    let realm_id = Uuid::new_v4();

    let draft: FlowDraft = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as name, ? as description, ? as graph_json, ? as flow_type, ? as created_at, ? as updated_at",
    )
    .bind(draft_id.to_string())
    .bind(realm_id.to_string())
    .bind("Draft")
    .bind("desc")
    .bind("[]")
    .bind("browser")
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("draft row");

    assert_eq!(draft.id, draft_id);
    assert_eq!(draft.realm_id, realm_id);

    let version: FlowVersion = sqlx::query_as(
        "SELECT ? as id, ? as flow_id, ? as version_number, ? as execution_artifact, ? as graph_json, ? as checksum, ? as created_at",
    )
    .bind("version-1")
    .bind("flow-1")
    .bind(1_i32)
    .bind("artifact")
    .bind("{}")
    .bind("checksum")
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("version row");

    assert_eq!(version.id, "version-1");
    assert_eq!(version.flow_id, "flow-1");

    let deployment_id = Uuid::new_v4();
    let active_version_id = Uuid::new_v4();
    let deployment: FlowDeployment = sqlx::query_as(
        "SELECT ? as id, ? as realm_id, ? as flow_type, ? as active_version_id, ? as updated_at",
    )
    .bind(deployment_id.to_string())
    .bind(realm_id.to_string())
    .bind("browser")
    .bind(active_version_id.to_string())
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("deployment row");

    assert_eq!(deployment.realm_id, realm_id);
    assert_eq!(deployment.active_version_id, active_version_id.to_string());
}
