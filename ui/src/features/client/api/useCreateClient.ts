import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

interface CreateClientPayload {
  client_id: string
  redirect_uris: string[]
  web_origins: string[]
}

export function useCreateClient() {
  const queryClient = useQueryClient()
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: CreateClientPayload) => {
      // POST /api/realms/{realm}/clients
      return apiClient.post(`/api/realms/${realm}/clients`, data)
    },
    onSuccess: () => {
      toast.success('Client created successfully')
      void queryClient.invalidateQueries({ queryKey: ['clients', realm] })
      navigate('/clients')
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to create client')
    },
  })
}
