import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { DeleteIdentityProviderResult } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface DeleteIdentityProviderPayload {
  hard?: boolean
}

export function useDeleteIdentityProvider(providerId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ hard }: DeleteIdentityProviderPayload = {}) => {
      const query = hard ? '?hard=true' : ''
      return apiClient.delete<DeleteIdentityProviderResult>(
        `/api/realms/${realm}/identity-providers/${providerId}${query}`,
      )
    },
    onSuccess: (result) => {
      if (result.outcome === 'soft_deleted') {
        toast.success(
          result.linked_identity_count > 0
            ? `Provider disabled and preserved ${result.linked_identity_count} linked identities.`
            : 'Provider disabled.',
        )
      } else {
        toast.success(
          result.linked_identity_count > 0
            ? `Provider and ${result.linked_identity_count} linked identities deleted.`
            : 'Provider deleted.',
        )
      }
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProviders(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.identityProvider(realm, providerId) })
    },
    onError: (err: unknown) => {
      const message = err instanceof Error ? err.message : 'Failed to delete identity provider'
      toast.error(message)
    },
  })
}
