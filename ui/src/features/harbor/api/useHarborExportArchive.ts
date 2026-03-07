import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { exportHarborArchive } from '@/features/harbor/api/harborApi'

export function useHarborExportArchive() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: {
      scope: string
      id?: string
      selection?: string[]
      includeSecrets?: boolean
      archiveFormat?: string
      asyncMode?: boolean
    }) => {
      if (!realm) throw new Error('Missing realm')
      return await exportHarborArchive({ realm, ...params })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to export Harbor bundle')
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['harbor-jobs', realm] })
    },
  })
}
