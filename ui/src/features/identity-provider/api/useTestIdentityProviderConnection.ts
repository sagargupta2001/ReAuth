import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { IdentityProviderConnectionTestResult } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useTestIdentityProviderConnection(providerId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: () =>
      apiClient.post<IdentityProviderConnectionTestResult>(
        `/api/realms/${realm}/identity-providers/${providerId}/test-connection`,
        {},
      ),
    onSuccess: (result) => {
      if (result.ok) {
        toast.success('Connection test passed.')
      } else {
        toast.error('Connection test found provider issues.')
      }
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProvider(realm, providerId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProviders(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to test provider connection.')
    },
  })
}
