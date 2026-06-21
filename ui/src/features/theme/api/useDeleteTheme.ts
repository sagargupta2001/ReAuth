import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useDeleteTheme(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async () => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return apiClient.delete(`/api/realms/${realm}/themes/${themeId}`)
    },
    onSuccess: () => {
      toast.success('Theme deleted.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to delete theme.')
    },
  })
}
