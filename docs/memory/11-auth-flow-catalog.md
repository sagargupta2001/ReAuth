# Auth Flow Catalog

This file lists the known flow types and their current templates. All flows execute via the graph engine (FlowExecutor).

## Where flows are defined
- Templates (graph JSON): `/src/application/flow_manager/templates.rs`
- Flow metadata: `/src/domain/auth_flow.rs`
- Flow bindings (realm slots): `/src/domain/realm.rs`
- Publish/bind logic: `/src/application/flow_manager/mod.rs`

## browser (default login)
- Template: `FlowTemplates::browser_flow()`
- Nodes: `core.start` -> `core.auth.cookie` -> `core.logic.condition` (SSO) -> `core.auth.password` -> `core.logic.condition` (OIDC) -> `core.oidc.consent` -> `core.terminal.allow`
- Purpose: standard interactive login with SSO-cookie check first and optional OIDC consent.
- Consent is only triggered when `oidc.client_id` is present in session context.
- Binding slot: `browser_flow_id` in realm.

## direct (direct grant)
- Template: `FlowTemplates::direct_grant_flow()`
- Nodes: `core.auth.password` -> `core.terminal.allow`
- Purpose: non-UI direct login (currently the same password node as UI).
- Binding slot: `direct_grant_flow_id` in realm.

## registration
- Template: `FlowTemplates::registration_flow()`
- Nodes: `core.start` -> `core.auth.register` -> `core.terminal.allow`
- Purpose: self-service registration with role assignment and realm policies.
- Binding slot: `registration_flow_id` in realm.

## reset
- Template: `FlowTemplates::reset_credentials_flow()`
- Nodes: `core.start` -> `core.auth.forgot_credentials` -> `core.auth.reset_password` -> `core.terminal.allow`
- Purpose: recovery request + reset password flow.
- Binding slot: `reset_credentials_flow_id` in realm.

## oidc-consent (node)
- Node type: `core.oidc.consent`
- Purpose: capture user approval/denial of requested OIDC scopes.
- Outputs: `allow` (continue flow) and `deny` (terminate with failure).
- Default UI template: `consent` (Fluid).

## Reserved (not fully wired yet)
The publish logic recognizes these flow types but realm schema does not yet have columns for them.
- `client` -> tries to bind to `client_authentication_flow_id`
- `docker` -> tries to bind to `docker_authentication_flow_id`

Adding these requires a schema update to `realms` plus domain/DTO updates.

## Adding a new flow type (current code)
1. Add a template to `FlowTemplates` that returns a valid graph JSON.
2. Ensure the flow `type` matches the template key used by `generate_default_graph`.
3. (Optional) Add a realm binding column and update `update_flow_binding` whitelist.
