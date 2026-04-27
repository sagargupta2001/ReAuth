import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { OidcClient } from '@/entities/oidc/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

interface CreateClientPayload {
  client_id: string
  redirect_uris: string[]
  web_origins: string[]
}

export function useCreateClient() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: CreateClientPayload) => {
      // POST /api/realms/{realm}/clients
      return apiClient.post<OidcClient>(`/api/realms/${realm}/clients`, data)
    },
    onSuccess: () => {
      toast.success('Client created successfully')
      void queryClient.invalidateQueries({ queryKey: queryKeys.clients(realm) })
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to create client')
    },
  })
}
