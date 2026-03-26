import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ThemeDetails } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export interface CreateThemePayload {
  name: string
  description?: string
}

export function useCreateTheme() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: CreateThemePayload) => {
      if (!realm) throw new Error('Missing Realm')
      return await apiClient.post<ThemeDetails>(`/api/realms/${realm}/themes`, payload)
    },
    onSuccess: () => {
      toast.success('Theme created')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to create theme'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
