import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { Invitation, InvitationStatus, CreateInvitationRequest } from '@/entities/invitation/model/types'
import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface InvitationSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  status?: InvitationStatus
}

export interface AcceptInvitationRequest {
  token: string
  username: string
  password: string
}

export interface AcceptInvitationResponse {
  status: 'redirect'
  url: string
}

export function useInvitations(params: InvitationSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.invitations(realm, params),
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)
      if (params.sort_by) query.set('sort_by', params.sort_by)
      if (params.sort_dir) query.set('sort_dir', params.sort_dir)
      if (params.status) query.set('status', params.status)

      return apiClient.get<PaginatedResponse<Invitation>>(
        `/api/realms/${realm}/invitations?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
  })
}

export function useCreateInvitation() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: CreateInvitationRequest) =>
      apiClient.post<Invitation>(`/api/realms/${realm}/invitations`, payload),
    onSuccess: () => {
      toast.success('Invitation sent successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.invitations() })
    },
  })
}

export function useResendInvitation() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (invitationId: string) =>
      apiClient.post<Invitation>(`/api/realms/${realm}/invitations/${invitationId}/resend`, {}),
    onSuccess: () => {
      toast.success('Invitation resent successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.invitations() })
    },
  })
}

export function useRevokeInvitation() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (invitationId: string) =>
      apiClient.post<Invitation>(`/api/realms/${realm}/invitations/${invitationId}/revoke`, {}),
    onSuccess: () => {
      toast.success('Invitation revoked.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.invitations() })
    },
  })
}

export function useAcceptInvitation(realm: string) {
  return useMutation({
    mutationFn: (payload: AcceptInvitationRequest) =>
      apiClient.post<AcceptInvitationResponse>(`/api/realms/${realm}/invitations/accept`, payload),
  })
}
