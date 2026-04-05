import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface NodeCapabilities {
  supports_ui: boolean
  ui_surface?: 'form' | 'awaiting_action' | null
  allowed_page_categories?: Array<
    'auth' | 'consent' | 'awaiting_action' | 'verification' | 'mfa' | 'notification' | 'error' | 'custom'
  >
  async_pause?: boolean
  side_effects?: boolean
  requires_secrets?: boolean
}

export interface NodeContract {
  id: string
  category: string
  display_name: string
  description: string
  icon: string
  inputs: string[]
  outputs: string[]
  config_schema: Record<string, unknown>
  default_template_key?: string | null
  contract_version?: string
  capabilities: NodeCapabilities
}

export function useNodes() {
  const realm = useActiveRealm()
  return useQuery({
    queryKey: queryKeys.flowNodes(realm),
    queryFn: async () => {
      return apiClient.get<NodeContract[]>(`/api/realms/${realm}/flows/nodes`)
    },
    staleTime: 1000 * 60 * 5, // Cache for 5 mins
  })
}
