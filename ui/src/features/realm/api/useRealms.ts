import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client.ts'

import type { Realm } from '../../../entities/realm/model/types.ts'

export function useRealms() {
  return useQuery({
    queryKey: ['realms'],
    // No arguments needed! The client handles the token internally.
    queryFn: () => apiClient.get<Realm[]>('/api/realms'),
  })
}
