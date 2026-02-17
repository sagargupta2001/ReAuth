import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { apiClient } from '@/shared/api/client.ts'

interface CreateUserPayload {
  username: string
  password?: string // Optional if you implement password generation
}

export function useCreateUser() {
  const queryClient = useQueryClient()
  const realm = useActiveRealm()

  const navigate = useRealmNavigate()

  return useMutation({
    mutationFn: (data: CreateUserPayload) => {
      return apiClient.post(`/api/realms/${realm}/users`, data)
    },
    onSuccess: () => {
      toast.success('User created successfully')
      void queryClient.invalidateQueries({ queryKey: ['users'] })
      navigate('/users')
    },
    onError: (error) => {
      toast.error(error.message || 'Failed to create user')
    },
  })
}
