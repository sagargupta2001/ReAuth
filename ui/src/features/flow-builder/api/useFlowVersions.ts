import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface FlowVersion {
  id: string
  version_number: number
  created_at: string
}

export function useFlowVersions(flowId: string) {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: ['flow-versions', flowId],
    queryFn: async () => {
      return await apiClient.get<FlowVersion[]>(`/api/realms/${realm}/flows/${flowId}/versions`)
    },
    enabled: !!flowId,
  })
}
