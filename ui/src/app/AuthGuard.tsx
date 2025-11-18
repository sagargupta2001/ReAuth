import { useEffect, useRef, useState } from 'react'

import { useQuery } from '@tanstack/react-query'
import { Navigate, useLocation } from 'react-router-dom'

import { useSessionStore } from '@/entities/session/model/sessionStore'
import { refreshAccessToken } from '@/features/auth/api/authApi'

// Helper: Starts the login flow if we are definitely logged out
const LoginFlowInitiator = () => {
  const location = useLocation()

  const { data, isLoading, isError } = useQuery({
    queryKey: ['startLoginFlow'],
    queryFn: async () => {
      const res = await fetch('/api/auth/login')
      if (!res.ok) throw new Error('Failed to start login flow')
      return res.json()
    },
    retry: false,
  })

  if (isLoading)
    return <div className="flex h-screen items-center justify-center">Initializing login...</div>

  if (isError || !data?.challenge_page) {
    return (
      <div className="flex h-screen items-center justify-center text-red-500">
        Authentication service unavailable.
      </div>
    )
  }

  return (
    <Navigate
      to={{
        pathname: data.challenge_page,
        search: `?redirect=${location.pathname}`,
      }}
      replace
    />
  )
}

export const AuthGuard = ({ children }: { children: React.ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const [isRestoringSession, setIsRestoringSession] = useState(true)
  const isInitialized = useRef(false)

  useEffect(() => {
    const restoreSession = async () => {
      // 1. If we already have a token, we are good.
      if (accessToken || isInitialized.current) {
        setIsRestoringSession(false)
        return
      }

      isInitialized.current = true

      // 2. If not, try to refresh using the HttpOnly cookie
      try {
        console.log('[AuthGuard] Attempting to restore session via refresh token...')
        const newToken = await refreshAccessToken()
        setSession(newToken) // This decodes the JWT and sets the user
        console.log('[AuthGuard] Session restored successfully.')
      } catch (err) {
        console.log('[AuthGuard] Session restoration failed (user likely logged out).')
        // This is fine, it just means the user needs to log in
      } finally {
        setIsRestoringSession(false)
      }
    }

    restoreSession()
  }, [accessToken, setSession])

  // --- RENDER STATES ---

  if (isRestoringSession) {
    // Show a loading screen while checking cookies
    return <div className="flex h-screen items-center justify-center">Checking session...</div>
  }

  if (accessToken) {
    // User is authenticated (either preserved or restored)
    return <>{children}</>
  }

  // Restoration failed, user is strictly unauthenticated
  return <LoginFlowInitiator />
}
