import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

interface UpdateClientPayload {
  client_id: string
  redirect_uris: string[]
  web_origins: string[]
}

export function useUpdateClient(id: string) {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  return useMutation({
    mutationFn: (data: UpdateClientPayload) => {
      return apiClient.put(`/api/realms/${realm}/clients/${id}`, data)
    },
    onSuccess: () => {
      toast.success('Client updated successfully')
      void queryClient.invalidateQueries({ queryKey: ['client', realm, id] })
      void queryClient.invalidateQueries({ queryKey: ['clients'] })
    },
  })
}
