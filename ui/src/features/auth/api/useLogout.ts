import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useLocation, useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { authApi } from '@/features/auth/api/authApi.ts'

export function useLogout() {
  const navigate = useNavigate()
  const location = useLocation()
  const clearSession = useSessionStore((state) => state.clearSession)
  const queryClient = useQueryClient()

  // 2. Get the active realm (default to 'master' if undefined)
  const realm = useActiveRealm() || 'master'

  return useMutation({
    // 3. Inject the realm into the API call
    mutationFn: async () => {
      return authApi.logout(realm)
    },
    onSuccess: () => {
      // Clear client state
      clearSession()
      queryClient.clear()

      // 4. Construct Return URL with Realm Context
      // We add ?realm=... so the login screen initializes the correct flow immediately
      const currentPath = location.pathname + location.search
      const loginUrl = `/login?realm=${realm}&redirect=${encodeURIComponent(currentPath)}`

      navigate(loginUrl, { replace: true })

      toast.success('Logged out successfully')
    },
    onError: (error) => {
      // Force client-side cleanup even if server fails (Self-Healing)
      clearSession()
      queryClient.clear()

      const currentPath = location.pathname + location.search
      const loginUrl = `/login?realm=${realm}&redirect=${encodeURIComponent(currentPath)}`

      navigate(loginUrl, { replace: true })

      console.error('Logout failed on server:', error)
    },
  })
}
