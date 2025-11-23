import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { useSessionStore } from '@/entities/session/model/sessionStore'

import { authApi } from './authApi'

export function useLogout() {
  const navigate = useNavigate()
  const clearSession = useSessionStore((state) => state.clearSession)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: authApi.logout,
    onSuccess: () => {
      // Clear client state
      clearSession()

      // Clear any cached queries (optional but recommended)
      queryClient.clear()

      // Redirect to login page
      navigate('/login', { replace: true })

      toast.success('Logged out successfully')
    },
    onError: (error) => {
      // Even if the server fails, we should force a client-side logout
      clearSession()
      navigate('/login', { replace: true })
      console.error('Logout failed on server:', error)
    },
  })
}
