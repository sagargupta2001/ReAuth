export type IdentityProviderProtocol = 'oidc' | 'oauth2'

export interface IdentityProvider {
  id: string
  realm_id: string
  alias: string
  display_name: string
  protocol: IdentityProviderProtocol
  preset_key?: string | null
  enabled: boolean
  client_id: string
  issuer?: string | null
  authorization_endpoint?: string | null
  token_endpoint?: string | null
  userinfo_endpoint?: string | null
  jwks_uri?: string | null
  scopes: string[]
  claim_mapping: Record<string, unknown>
  pkce_required: boolean
  allow_login: boolean
  allow_link: boolean
  allow_jit_provisioning: boolean
  allow_email_auto_link: boolean
  require_verified_email: boolean
  icon_ref?: string | null
  button_color?: string | null
  sort_order: number
  metadata_cached_at?: string | null
  jwks_cached_at?: string | null
  client_secret_set: boolean
  client_secret_mask?: string | null
}

export interface DeleteIdentityProviderResult {
  provider_id: string
  provider_alias: string
  outcome: 'soft_deleted' | 'hard_deleted'
  linked_identity_count: number
}

export interface IdentityProviderConnectionCheck {
  attempted: boolean
  ok: boolean
  status_code?: number | null
  detail: string
}

export interface IdentityProviderConnectionTestResult {
  provider_id: string
  provider_alias: string
  protocol: IdentityProviderProtocol
  ok: boolean
  discovery: IdentityProviderConnectionCheck
  token_endpoint: IdentityProviderConnectionCheck
  userinfo_endpoint: IdentityProviderConnectionCheck
  jwks: IdentityProviderConnectionCheck
  metadata_cached_at?: string | null
  jwks_cached_at?: string | null
  tested_at: string
}

export interface IdentityProviderLinkedUser {
  federated_identity_id: string
  user_id: string
  username: string
  email?: string | null
  subject: string
  external_username?: string | null
  external_email?: string | null
  linked_via: string
  linked_at: string
  last_provider_login_at?: string | null
  last_user_sign_in_at?: string | null
}

export interface IdentityProviderActivitySummary {
  total_events_last_24h: number
  failures_last_24h: number
  callback_success_last_24h: number
  links_last_24h: number
  jit_provisioned_last_24h: number
}

export interface IdentityProviderActivityEvent {
  audit_event_id: string
  action: string
  created_at: string
  actor_user_id?: string | null
  auth_session_id?: string | null
  user_id?: string | null
  subject?: string | null
  email?: string | null
  linked_via?: string | null
  message?: string | null
  metadata: Record<string, unknown>
}

export interface IdentityProviderActivityFeed {
  provider_id: string
  provider_alias: string
  summary: IdentityProviderActivitySummary
  recent_events: IdentityProviderActivityEvent[]
}

export interface IdentityProviderPreset {
  key: string
  display_name: string
  protocol: IdentityProviderProtocol
  issuer?: string | null
  authorization_endpoint?: string | null
  token_endpoint?: string | null
  userinfo_endpoint?: string | null
  jwks_uri?: string | null
  scopes: string[]
  claim_mapping: Record<string, unknown>
  icon_ref?: string | null
}
