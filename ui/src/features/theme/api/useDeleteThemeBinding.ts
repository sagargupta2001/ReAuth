import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export function useDeleteThemeBinding(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (clientId: string) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return apiClient.delete(`/api/realms/${realm}/themes/${themeId}/bindings/${clientId}`)
    },
    onSuccess: (_data, clientId) => {
      toast.success('Client override removed')
      void queryClient.invalidateQueries({ queryKey: ['theme-bindings', realm, themeId] })
      void queryClient.invalidateQueries({
        queryKey: ['theme-bindings', 'client', realm, clientId],
      })
      void queryClient.invalidateQueries({ queryKey: ['theme-preview', realm, themeId] })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to remove client override'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
