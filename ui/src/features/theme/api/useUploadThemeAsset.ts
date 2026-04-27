import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ThemeAsset } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUploadThemeAsset(themeId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (file: File) => {
      if (!realm || !themeId) throw new Error('Missing Realm or Theme ID')
      const formData = new FormData()
      formData.append('file', file)
      return await apiClient.postForm<ThemeAsset>(
        `/api/realms/${realm}/themes/${themeId}/assets`,
        formData,
      )
    },
    onSuccess: () => {
      toast.success('Asset uploaded')
      void queryClient.invalidateQueries({ queryKey: queryKeys.themeAssets(realm, themeId) })
    },
    onError: (error: unknown) => {
      let msg = 'Failed to upload asset'
      if (error && typeof error === 'object' && 'response' in error) {
        const errObj = error as { response?: { data?: { error?: string } } }
        msg = errObj.response?.data?.error || msg
      }
      toast.error(msg)
    },
  })
}
