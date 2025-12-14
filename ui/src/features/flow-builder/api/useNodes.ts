import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface NodeMetadata {
  id: string
  category: string
  display_name: string
  description: string
  icon: string
  inputs: string[]
  outputs: string[]
  config_schema: any
}

export function useNodes() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: ['flow-nodes', realm],
    queryFn: async () => {
      return apiClient.get<NodeMetadata[]>(`/api/realms/${realm}/flows/nodes`)
    },
    staleTime: 1000 * 60 * 5, // Cache for 5 mins
  })
}
