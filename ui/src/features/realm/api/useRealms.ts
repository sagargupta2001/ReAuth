import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

import type { Realm } from '../../../entities/realm/model/types.ts'

export function useRealms() {
  return useQuery({
    queryKey: queryKeys.realms(),
    // No arguments needed! The client handles the token internally.
    queryFn: () => apiClient.get<Realm[]>('/api/realms'),
  })
}
