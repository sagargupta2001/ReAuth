# Data Model

Source of truth: `migrations/20251214045651_initial_schema.sql` and subsequent migrations.

## Core tables

### realms
- `id`, `name`
- Token TTLs: `access_token_ttl_secs`, `refresh_token_ttl_secs`
- Flow bindings: `browser_flow_id`, `registration_flow_id`, `direct_grant_flow_id`, `reset_credentials_flow_id`

### users
- `id`, `realm_id`, `username`, `first_name`, `last_name`, `hashed_password`, `created_at`, `updated_at`, `last_sign_in_at`, `locked_until`, `banned_at`
- Access status:
  - `locked_until`: temporary admin lock timestamp; future values block sign-in.
  - `banned_at`: indefinite admin ban timestamp; non-null values block sign-in.
- Metadata JSON text columns:
  - `public_metadata_json`: authenticated frontend-safe and backend/admin-readable user metadata.
  - `private_metadata_json`: backend/admin-only metadata; current v1 redaction is handled in the application response layer to allow future granular permissions.
  - `unsafe_metadata_json`: authenticated frontend-safe and backend/admin-readable/writable metadata.
- Uniqueness: `(realm_id, username)`

### user_emails / user_phone_numbers
- `user_emails`: multiple email addresses per user with `email`, `email_normalized`, `is_primary`, `is_verified`, timestamps.
- `user_phone_numbers`: multiple phone numbers per user with `phone_number`, `phone_number_normalized`, `is_primary`, `is_verified`, timestamps.
- Uniqueness is realm-scoped on normalized values:
  - `user_emails`: `(realm_id, email_normalized)`
  - `user_phone_numbers`: `(realm_id, phone_number_normalized)`
- Triggers enforce one primary email or phone number per user by demoting existing primary rows on insert/update.

### roles / groups
- `roles`: `id`, `realm_id`, optional `client_id`, `name`, `description`, `created_at`
- `groups`: `id`, `realm_id`, optional `parent_id`, `name`, `description`, `sort_order`, `created_at`
- Role name uniqueness: `(realm_id, client_id, name)`

### role_permissions / composites / mappings
- `role_permissions`: `(role_id, permission_name)`
- `role_composite_roles`: role inheritance
- `user_roles`, `group_roles`, `user_groups`

### oidc_clients
- `id`, `realm_id`, `client_id`, `client_secret`, `redirect_uris`, `web_origins`, `scopes`
- `client_id` is unique globally in schema (not per-realm)

### auth_flows (metadata)
- `id`, `realm_id`, `name`, `description`, `alias`, `type`, `built_in`

### flow_drafts / flow_versions / flow_deployments
- `flow_drafts`: editable graph JSON per realm
- `flow_versions`: compiled execution artifacts + graph JSON
- `flow_deployments`: active version pointer per `(realm_id, flow_type)`

### auth_sessions
- `id`, `realm_id`, `flow_version_id`, `current_node_id`, `context`, `status`, `user_id`, timestamps, `expires_at`
- Extra fields in schema (`execution_state`, `last_ui_output`) exist but are not currently used by runtime code.

### authorization_codes
- Authorization codes for OIDC, with PKCE fields and expiry.

### refresh_tokens
- Persistent refresh tokens for SSO/session management.

### seed_history
- Tracks applied seeders: `name`, `version`, `checksum`, `applied_at`.

## Indices
- Role/group and mapping indices exist for RBAC lookups.
- `idx_auth_sessions_expires` to clean or scan expiring sessions.

## Legacy tables removed
- `auth_flow_steps`, `authenticator_config`, `login_sessions` were removed in migration:
  - `migrations/20260215120000_remove_step_based_flows.sql`

## Notes
- Realm flow bindings are nullable to avoid circular FK constraints.
- Flow types like `client` and `docker` are referenced in code but schema does not yet include columns for them.
