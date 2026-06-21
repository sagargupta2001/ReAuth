import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useDeleteClient(clientId: string) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => apiClient.delete(`/api/realms/${realm}/clients/${clientId}`),
    onSuccess: () => {
      toast.success('Client deleted')
      void queryClient.invalidateQueries({ queryKey: queryKeys.clients() })
      void queryClient.invalidateQueries({ queryKey: queryKeys.clientStats(realm) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.client(realm, clientId) })
    },
    onError: (error: unknown) => {
      const message = error instanceof Error ? error.message : 'Failed to delete client'
      toast.error(message)
    },
  })
}
