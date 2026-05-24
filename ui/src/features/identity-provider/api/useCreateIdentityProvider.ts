import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { IdentityProvider } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { IdentityProviderPayload } from '@/features/identity-provider/lib/form'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useCreateIdentityProvider() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: IdentityProviderPayload) =>
      apiClient.post<IdentityProvider>(`/api/realms/${realm}/identity-providers`, data),
    onSuccess: () => {
      toast.success('Identity provider created successfully.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProviders(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to create identity provider.')
    },
  })
}
