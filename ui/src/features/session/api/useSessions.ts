import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Session } from '@/entities/session/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'
import { serializeFilterValue, type DataTableFilterValue } from '@/shared/ui/data-table/types'

const supportedSessionFilterKeys = new Set(['started'])

export interface SessionSearchParams {
  page?: number
  per_page?: number
  q?: string // Search by username or user ID
  filters?: DataTableFilterValue[]
}

interface RevokeCountResponse {
  count: number
}

function hasMeaningfulFilterValue(value: unknown): boolean {
  if (value == null) return false
  if (typeof value === 'string') return value.trim().length > 0
  if (typeof value === 'object') {
    const range = value as { from?: unknown; to?: unknown }
    return range.from != null || range.to != null
  }
  return true
}

export function useSessions(params: SessionSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.sessions(realm, params),
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)

      params.filters?.forEach((f) => {
        if (!supportedSessionFilterKeys.has(f.key)) return
        if (!hasMeaningfulFilterValue(f.value)) return
        const value = serializeFilterValue(f.value)
        if (!value) return
        query.set(`filter_${f.key}`, value)
      })

      return apiClient.get<PaginatedResponse<Session>>(
        `/api/realms/${realm}/sessions?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
    refetchInterval: 5000,
  })
}

function invalidateSessions(queryClient: ReturnType<typeof useQueryClient>) {
  void queryClient.invalidateQueries({ queryKey: queryKeys.sessions() })
}

export function useRevokeSession() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (sessionId: string) => {
      return apiClient.delete(`/api/realms/${realm}/sessions/${sessionId}`)
    },
    onSuccess: () => {
      toast.success('Session revoked successfully')
      invalidateSessions(queryClient)
    },
    onError: (err: unknown) => {
      // If the backend says "Not Found" or "Invalid Token", it means the session
      // was already rotated or deleted. We treat this as a UI sync update.
      const errorMessage = (err instanceof Error ? err.message : '').toLowerCase()

      if (errorMessage.includes('invalid') || errorMessage.includes('not found')) {
        toast.info('Session was already inactive or rotated.')
        invalidateSessions(queryClient)
        return
      }

      toast.error(`Failed to revoke session: ${errorMessage || 'Unknown error'}`)
    },
  })
}

/** Bulk-revoke an explicit set of sessions. The caller's current session is excluded server-side. */
export function useRevokeSessions() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (sessionIds: string[]) => {
      return apiClient.post<RevokeCountResponse>(`/api/realms/${realm}/sessions/revoke`, {
        scope: 'selected',
        session_ids: sessionIds,
      })
    },
    onSuccess: (res) => {
      toast.success(`Revoked ${res.count} session${res.count === 1 ? '' : 's'}`)
      invalidateSessions(queryClient)
    },
    onError: (err: unknown) => {
      toast.error(
        `Failed to revoke sessions: ${err instanceof Error ? err.message : 'Unknown error'}`,
      )
    },
  })
}

/** Revoke all of the caller's active sessions except the current one. */
export function useRevokeOtherSessions() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: () => {
      return apiClient.post<RevokeCountResponse>(`/api/realms/${realm}/sessions/revoke`, {
        scope: 'others',
      })
    },
    onSuccess: (res) => {
      toast.success(
        res.count > 0
          ? `Revoked ${res.count} other session${res.count === 1 ? '' : 's'}`
          : 'No other active sessions to revoke',
      )
      invalidateSessions(queryClient)
    },
    onError: (err: unknown) => {
      toast.error(
        `Failed to revoke other sessions: ${err instanceof Error ? err.message : 'Unknown error'}`,
      )
    },
  })
}

/** Revoke every active session belonging to a user (admin-wide eviction). */
export function useRevokeUserSessions() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (userId: string) => {
      return apiClient.post<RevokeCountResponse>(`/api/realms/${realm}/sessions/revoke`, {
        scope: 'user',
        user_id: userId,
      })
    },
    onSuccess: (res) => {
      toast.success(`Revoked ${res.count} session${res.count === 1 ? '' : 's'} for this user`)
      invalidateSessions(queryClient)
    },
    onError: (err: unknown) => {
      toast.error(
        `Failed to revoke user sessions: ${err instanceof Error ? err.message : 'Unknown error'}`,
      )
    },
  })
}

/** Force a session to re-authenticate on its next refresh (step-up). */
export function useStepUpSession() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (sessionId: string) => {
      return apiClient.post(`/api/realms/${realm}/sessions/${sessionId}/step-up`, {})
    },
    onSuccess: () => {
      toast.success('Re-authentication required on next request for this session')
      invalidateSessions(queryClient)
    },
    onError: (err: unknown) => {
      toast.error(
        `Failed to request re-authentication: ${
          err instanceof Error ? err.message : 'Unknown error'
        }`,
      )
    },
  })
}
