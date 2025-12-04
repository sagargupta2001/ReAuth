import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Session } from '@/entities/session/model/types'
import { apiClient } from '@/shared/api/client'

export interface SessionSearchParams {
  page?: number
  per_page?: number
  q?: string // Search by User ID
}

export function useSessions(params: SessionSearchParams) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['sessions', realm, params],
    queryFn: async () => {
      const query = new URLSearchParams()
      query.set('page', String(params.page || 1))
      query.set('per_page', String(params.per_page || 10))
      if (params.q) query.set('q', params.q)

      return apiClient.get<PaginatedResponse<Session>>(
        `/api/realms/${realm}/sessions?${query.toString()}`,
      )
    },
    placeholderData: keepPreviousData,
    refetchInterval: 5000,
  })
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
      // Invalidate list to refresh UI
      void queryClient.invalidateQueries({ queryKey: ['sessions'] })
    },
    onError: (err: any) => {
      // If the backend says "Not Found" or "Invalid Token", it means the session
      // was already rotated or deleted. We should treat this as a UI sync update, not an error.
      // Note: Check what status code your backend returns for missing session (404 or 401)

      const errorMessage = err.message?.toLowerCase() || ''

      if (errorMessage.includes('invalid') || errorMessage.includes('not found')) {
        toast.info('Session was already inactive or rotated.')
        // Refresh the list to show the NEW session ID
        void queryClient.invalidateQueries({ queryKey: ['sessions'] })
        return
      }

      // Genuine errors (e.g. 500 server error)
      toast.error(`Failed to revoke session: ${err.message}`)
    },
  })
}
