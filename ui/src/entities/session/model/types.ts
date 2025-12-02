export interface Session {
  id: string
  user_id: string
  realm_id: string
  client_id?: string
  ip_address?: string
  user_agent?: string
  created_at: string // ISO Date String
  last_used_at: string // ISO Date String
  expires_at: string // ISO Date String
}

// Re-export shared types for convenience
export type { PaginatedResponse } from '@/entities/oidc/model/types'
