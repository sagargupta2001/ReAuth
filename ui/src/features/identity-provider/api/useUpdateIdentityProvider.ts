import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { IdentityProvider } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { IdentityProviderPayload } from '@/features/identity-provider/lib/form'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUpdateIdentityProvider(providerId: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: IdentityProviderPayload) =>
      apiClient.put<IdentityProvider>(`/api/realms/${realm}/identity-providers/${providerId}`, data),
    onSuccess: () => {
      toast.success('Identity provider updated successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProvider(realm, providerId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProviders(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update identity provider.')
    },
  })
}
