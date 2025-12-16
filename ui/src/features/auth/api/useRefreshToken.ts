import { useMutation } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'

import { authApi } from './authApi'

export function useRefreshToken() {
  // 1. Get the current active realm (e.g., from URL params)
  const activeRealm = useActiveRealm()

  return useMutation({
    // 2. Wrap the API call to inject the realm
    mutationFn: async () => {
      // Fallback to 'master' if the hook returns undefined (e.g. at root path)
      const targetRealm = activeRealm || 'master'

      // Pass the realm to the API
      return authApi.refreshAccessToken(targetRealm)
    },
  })
}
