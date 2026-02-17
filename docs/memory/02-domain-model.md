# Domain Model

This summarizes domain structs/enums in `reauth/crates/reauth_core/src/domain`.

## Identity and tenancy
- Realm: `id`, `name`, token TTLs, and flow bindings (`browser_flow_id`, `registration_flow_id`, `direct_grant_flow_id`, `reset_credentials_flow_id`).
- User: `id`, `realm_id`, `username`, `hashed_password`.

## RBAC and permissions
- Role: `id`, `realm_id`, optional `client_id`, `name`, `description`.
- Group: `id`, `realm_id`, `name`, `description`.
- Permission (alias): `String`.
- PermissionDef: UI metadata for a permission (`id`, `name`, `description`).
- ResourceGroup: groups permissions for UI display (`id`, `label`, `description`, `permissions`).
- System permission registry: constants like `realm:read`, `user:write`, `rbac:write`, `event:read`, `session:revoke`, plus wildcard `*`.

## Sessions and auth state
- AuthenticationSession: tracks flow execution state with `realm_id`, `flow_version_id`, `current_node_id`, `context`, `status`, optional `user_id`, timestamps, and `expires_at`.
- SessionStatus: `Active`, `Completed`, `Failed` (serialized as lowercase).
- RefreshToken: session token record with `user_id`, `realm_id`, optional `client_id`, timestamps, and metadata.

## OIDC
- OidcClient: `realm_id`, `client_id`, optional `client_secret`, `redirect_uris`, `scopes`, `web_origins` (stored as JSON array strings).
- AuthCode: `code`, `user_id`, `client_id`, `redirect_uri`, optional `nonce`, optional PKCE fields, and `expires_at`.
- OidcContext: normalized OIDC request context (client_id, redirect_uri, response_type, scope, state, nonce, PKCE).
- OidcRequest: raw OIDC request payload (same fields, `Deserialize`).

## Flow system (graph-based)
- FlowDraft: editable flow graph with `graph_json`, `flow_type`, timestamps.
- FlowVersion: immutable compiled flow with `execution_artifact`, `graph_json`, `checksum`, `version_number`.
- FlowDeployment: active pointer by `realm_id` and `flow_type` to `active_version_id`.
- NodeMetadata: UI palette definitions (`id`, `category`, `display_name`, `description`, `icon`, `config_schema`, `inputs`, `outputs`).
- NodeProvider (trait): contract for node definitions used by the builder (id, name, description, icon, category, inputs/outputs, config schema).

## Flow execution
- ExecutionPlan: compiled flow with `start_node_id` and `nodes` map.
- ExecutionNode: `id`, `step_type`, `next` edges, and `config`.
- StepType: `Authenticator`, `Logic`, `Terminal`.
- ExecutionResult: `Challenge`, `Success`, `Failure`, and internal `Continue`.
- NodeOutcome: lifecycle outcomes (`Continue`, `SuspendForUI`, `SuspendForAsync`, `Reject`, `FlowSuccess`, `FlowFailure`).

## Auth flow metadata
- AuthFlow: `realm_id`, `name`, `description`, `alias`, `type`, `built_in`.
- Legacy step-based flow tables were removed in migration `20260215120000_remove_step_based_flows.sql`.

## Events
- DomainEvent: `UserCreated`, `UserAssignedToGroup`, `RoleAssignedToGroup`, `RolePermissionChanged`, `UserRoleAssigned`, `RoleDeleted`.

## Security primitives
- HashedPassword: Argon2id hashed password wrapper with `new`, `from_hash`, and `verify`.

## Supporting types
- PageRequest, PageResponse, PageMeta, SortDirection for pagination.

## Relationships (derived from fields)
- Most entities are scoped by `realm_id` (Realm, User, Role, Group, Auth/OIDC records, Flow draft/deployments).
- Flows: drafts are compiled into versions; deployments point a realm+flow_type to an active version.
- AuthenticationSession is tied to a flow version and current node.
- Roles may optionally be scoped to a client (`client_id`).

## Invariants and validation
- Flow graphs require exactly one start node and no dead-ends for non-terminal nodes (validated in compiler).
- AuthenticationSession expires and carries a mutable JSON `context`.
