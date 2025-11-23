import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'

import type { Realm } from '../model/types'

export function useRealms() {
  return useQuery({
    queryKey: ['realms'],
    // No arguments needed! The client handles the token internally.
    queryFn: () => apiClient.get<Realm[]>('/api/realms'),
  })
}
