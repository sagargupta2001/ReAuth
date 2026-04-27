import { useQuery } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'
import { SETUP_SEALED_STORAGE_KEY } from '@/shared/config/setup'
import { queryKeys } from '@/shared/lib/queryKeys'

export type SetupStatus = {
  required: boolean
}

export function useSetupStatus(options?: { enabled?: boolean }) {
  return useQuery<SetupStatus>({
    queryKey: queryKeys.setupStatus(),
    queryFn: async () => {
      const sealedCached = localStorage.getItem(SETUP_SEALED_STORAGE_KEY) === '1'
      try {
        const data = await apiClient.get<{ required?: boolean }>('/api/system/setup/status')
        const required = Boolean(data.required)
        if (required) {
          localStorage.removeItem(SETUP_SEALED_STORAGE_KEY)
        } else {
          localStorage.setItem(SETUP_SEALED_STORAGE_KEY, '1')
        }
        return { required }
      } catch (err) {
        if (sealedCached) {
          return { required: false }
        }
        throw err
      }
    },
    enabled: options?.enabled ?? true,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
  })
}
