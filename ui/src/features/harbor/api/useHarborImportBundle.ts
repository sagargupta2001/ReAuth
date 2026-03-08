import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import {
  importHarborArchive,
  importHarborBundle,
  isHarborAsyncResponse,
  summarizeImportResult,
  type HarborImportResponse,
} from '@/features/harbor/api/harborApi'

export function useHarborImportBundle() {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: {
      file: File
      scope: string
      id?: string
      conflictPolicy: string
      dryRun: boolean
      asyncMode?: boolean
    }) => {
      if (!realm) throw new Error('Missing realm')
      const fileName = params.file.name.toLowerCase()

      if (fileName.endsWith('.json')) {
        const raw = await params.file.text()
        let bundle: unknown
        try {
          bundle = JSON.parse(raw)
        } catch {
          throw new Error('Invalid JSON bundle')
        }

        return await importHarborBundle({
          realm,
          scope: params.scope,
          id: params.id,
          bundle,
          conflictPolicy: params.conflictPolicy,
          dryRun: params.dryRun,
          asyncMode: params.asyncMode,
        })
      }

      return await importHarborArchive({
        realm,
        scope: params.scope,
        id: params.id,
        file: params.file,
        conflictPolicy: params.conflictPolicy,
        dryRun: params.dryRun,
        asyncMode: params.asyncMode,
      })
    },
    onSuccess: (result: HarborImportResponse) => {
      if (isHarborAsyncResponse(result)) {
        toast.message('Import queued', {
          description: `Job ${result.job_id} is processing.`,
        })
      } else {
        const summary = summarizeImportResult(result)
        toast.success('Import complete', {
          description: `${summary.created} created · ${summary.updated} updated`,
        })
      }
      void queryClient.invalidateQueries({ queryKey: ['harbor-jobs', realm] })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to import Harbor bundle')
    },
  })
}
