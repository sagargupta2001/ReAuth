export interface Realm {
  id: string
  name: string
  access_token_ttl_secs: number
  refresh_token_ttl_secs: number
  pkce_required_public_clients: boolean
  lockout_threshold: number
  lockout_duration_secs: number
  is_system: boolean
  registration_enabled: boolean
  default_registration_role_ids: string[]
  browser_flow_id?: string | null
  registration_flow_id?: string | null
  direct_grant_flow_id?: string | null
  reset_credentials_flow_id?: string | null
}

export interface RealmEmailSettings {
  realm_id: string
  enabled: boolean
  from_address?: string | null
  from_name?: string | null
  reply_to_address?: string | null
  smtp_host?: string | null
  smtp_port?: number | null
  smtp_username?: string | null
  smtp_security: 'starttls' | 'tls' | 'none'
  smtp_password_set: boolean
}

export interface RealmRecoverySettings {
  realm_id: string
  token_ttl_minutes: number
  rate_limit_max: number
  rate_limit_window_minutes: number
  revoke_sessions_on_reset: boolean
  email_subject?: string | null
  email_body?: string | null
}

export interface RealmPasskeySettings {
  realm_id: string
  enabled: boolean
  allow_password_fallback: boolean
  discoverable_preferred: boolean
  challenge_ttl_secs: number
  reauth_max_age_secs: number
}

export interface RealmPasskeyAnalytics {
  realm_id: string
  window_hours: number
  credentials_total: number
  credentials_created_last_7d: number
  credentials_active_last_30d: number
  challenges: {
    pending_total: number
    pending_expired: number
  }
  outcomes: {
    assertion_success: number
    assertion_invalid_signature: number
    assertion_counter_regression: number
    assertion_challenge_mismatch: number
    enrollment_success: number
    enrollment_challenge_mismatch: number
  }
  recent_failures: Array<{
    action: string
    created_at: string
    actor_user_id?: string | null
    target_id?: string | null
  }>
}

export interface RealmSecurityHeaders {
  realm_id: string
  x_frame_options?: string | null
  content_security_policy?: string | null
  x_content_type_options?: string | null
  referrer_policy?: string | null
  strict_transport_security?: string | null
}
