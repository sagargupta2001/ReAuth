import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface InvitationStats {
  total: number
  pending: number
  accepted: number
}

export function useInvitationStats() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.invitationStats(realm),
    queryFn: () => apiClient.get<InvitationStats>(`/api/realms/${realm}/invitations/stats`),
  })
}
