import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { OidcClient } from '@/entities/oidc/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useRotateClientSecret(id: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: () =>
      apiClient.post<OidcClient>(`/api/realms/${realm}/clients/${id}/rotate-secret`, {}),
    onSuccess: () => {
      toast.success('Client secret rotated')
      void queryClient.invalidateQueries({ queryKey: queryKeys.client(realm, id) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.clients(realm) })
    },
    onError: (error: Error) => {
      toast.error(error.message || 'Failed to rotate client secret')
    },
  })
}
