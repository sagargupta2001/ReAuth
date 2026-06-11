import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import type {
  JsonObject,
  UserMetadata,
  UserMetadataVisibility,
} from '@/entities/user/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useUserMetadata(userId: string) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.userMetadata(userId),
    queryFn: () => apiClient.get<UserMetadata>(`/api/realms/${realm}/users/${userId}/metadata`),
    enabled: Boolean(userId),
  })
}

export function useUpdateUserMetadata(userId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      visibility,
      metadata,
    }: {
      visibility: UserMetadataVisibility
      metadata: JsonObject
    }) =>
      apiClient.put<UserMetadata>(`/api/realms/${realm}/users/${userId}/metadata/${visibility}`, {
        metadata,
      }),
    onSuccess: () => {
      toast.success('Metadata updated.')
      void queryClient.invalidateQueries({ queryKey: queryKeys.userMetadata(userId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.user(userId) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to update metadata.')
    },
  })
}
