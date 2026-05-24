import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { IdentityProvider } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useRefreshIdentityProviderMetadata(providerId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: () =>
      apiClient.post<IdentityProvider>(
        `/api/realms/${realm}/identity-providers/${providerId}/refresh-metadata`,
        {},
      ),
    onSuccess: () => {
      toast.success('Provider metadata refreshed.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProvider(realm, providerId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProviders(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to refresh provider metadata.')
    },
  })
}
