import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { ThemeSettingsSchema } from '@/features/theme/model/settings-schema'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUpdateTheme(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (values: ThemeSettingsSchema) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      return await apiClient.put(`/api/realms/${realm}/themes/${themeId}`, {
        name: values.name,
        description: values.description,
      })
    },
    onSuccess: () => {
      toast.success('Theme settings updated')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themes(realm, themeId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.themePreview(realm, themeId) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to update theme'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
