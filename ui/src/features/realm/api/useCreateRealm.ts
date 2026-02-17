import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import type { Realm } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'

interface CreateRealmPayload {
  name: string
}

export function useCreateRealm() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  return useMutation({
    mutationFn: (payload: CreateRealmPayload) => {
      return apiClient.post<Realm>('/api/realms', payload)
    },
    onSuccess: (newRealm) => {
      // Invalidate the list cache so the Realm Switcher updates immediately
      queryClient.invalidateQueries({ queryKey: ['realms'] })

      // This automatically updates the `useActiveRealm` hook and loads the dashboard.
      navigate(`/${newRealm.name}`)
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to create realm.')
    },
  })
}
