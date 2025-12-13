import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useLocation, useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { useSessionStore } from '@/entities/session/model/sessionStore'
import { authApi } from '@/features/auth/api/authApi.ts'

export function useLogout() {
  const navigate = useNavigate()
  const location = useLocation() // 1. Hook to get current path
  const clearSession = useSessionStore((state) => state.clearSession)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: authApi.logout,
    onSuccess: () => {
      // Clear client state
      clearSession()
      queryClient.clear()

      // 2. Construct Return URL
      // We capture where the user was (e.g. "/master/flows") so they can return there.
      const currentPath = location.pathname + location.search
      const loginUrl = `/login?redirect=${encodeURIComponent(currentPath)}`

      // 3. Navigate with Redirect Param
      navigate(loginUrl, { replace: true })

      toast.success('Logged out successfully')
    },
    onError: (error) => {
      // Force logout on error too (Self-Healing)
      clearSession()

      const currentPath = location.pathname + location.search
      navigate(`/login?redirect=${encodeURIComponent(currentPath)}`, { replace: true })

      console.error('Logout failed on server:', error)
    },
  })
}
