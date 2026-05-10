export const invitationStatuses = ['pending', 'accepted', 'expired', 'revoked'] as const

export type InvitationStatus = (typeof invitationStatuses)[number]

export interface Invitation {
  id: string
  email: string
  status: InvitationStatus
  expiry_days: number
  expires_at: string
  resend_count: number
  last_sent_at?: string | null
  accepted_at?: string | null
  revoked_at?: string | null
  created_at: string
  updated_at: string
}

export interface CreateInvitationRequest {
  email: string
  expiry_days: number
}
