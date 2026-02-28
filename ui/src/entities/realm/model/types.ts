export interface Realm {
  id: string
  name: string
  access_token_ttl_secs: number
  refresh_token_ttl_secs: number
  pkce_required_public_clients: boolean
  lockout_threshold: number
  lockout_duration_secs: number
  browser_flow_id?: string | null
  registration_flow_id?: string | null
  direct_grant_flow_id?: string | null
  reset_credentials_flow_id?: string | null
}
