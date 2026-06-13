export interface Session {
  id: string
  family_id?: string // Refresh-token family the session belongs to
  user_id: string
  username?: string | null // Owning user's username (enriched server-side)
  email?: string | null // Owning user's email (enriched server-side)
  realm_id: string
  client_id?: string
  ip_address?: string
  user_agent?: string
  created_at: string // ISO Date String (token issued-at / iat)
  last_used_at: string // ISO Date String
  expires_at: string // ISO Date String (token exp)
  step_up_at?: string | null // ISO Date String — set when forced re-auth is pending
}

// Derived, presentational classifications (computed client-side; no backend field).
export type SessionType = 'browser' | 'oauth'

export type SessionStatus = 'current' | 'reauth_pending' | 'expiring_soon' | 'idle' | 'active'

// Re-export shared types for convenience
export type { PaginatedResponse } from '@/entities/oidc/model/types'
