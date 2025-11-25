import { type ReactNode, useEffect, useRef, useState } from 'react'

import { AlertCircle } from 'lucide-react'
import { Navigate, useLocation } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useOidcAuth } from '@/features/auth/api/useOidcAuth'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken.ts'
import { PKCE_STORAGE_KEY } from '@/shared/config/oidc'
import { generateCodeChallenge, generateCodeVerifier } from '@/shared/lib/pkce'

export const AuthGuard = ({ children }: { children: ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const location = useLocation()
  const [isProcessing, setIsProcessing] = useState(true)
  const initRan = useRef(false)

  const { authorize, exchangeToken } = useOidcAuth()
  const refreshTokenMutation = useRefreshToken()

  useEffect(() => {
    const handleAuth = async () => {
      // If we are already logged in (in memory), stop.
      if (accessToken) {
        setIsProcessing(false)
        initRan.current = false
        return
      }

      if (initRan.current) return
      initRan.current = true

      // Check for Auth Code (Callback from OIDC)
      const searchParams = new URLSearchParams(window.location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
        if (verifier) {
          // We use .mutateAsync so we can await it properly
          try {
            const data = await exchangeToken.mutateAsync({ code: authCode, verifier })

            setSession(data.access_token)
            const newUrl = window.location.pathname + window.location.hash
            window.history.replaceState({}, document.title, newUrl)
            sessionStorage.removeItem(PKCE_STORAGE_KEY)
          } catch (err) {
            console.error('Token exchange failed', err)
            // Fall through to login flow on error
          } finally {
            // Ensure we stop processing regardless of success/failure
            setIsProcessing(false)
          }
        } else {
          console.error('PKCE Verifier missing')
          setIsProcessing(false)
        }
        return
      }

      // SILENT REFRESH (Restore Session)
      // Before forcing a new login, check if we have a valid cookie
      try {
        console.log('[AuthGuard] Attempting silent refresh...')
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
        return
      } catch (e) {
        console.log('[AuthGuard] Silent refresh failed.')
      }

      // Start OIDC Flow
      const verifier = generateCodeVerifier()
      const challenge = await generateCodeChallenge(verifier)
      sessionStorage.setItem(PKCE_STORAGE_KEY, verifier)

      try {
        const response = await authorize.mutateAsync(challenge)
        if (response.status === 'challenge' && response.challenge_page) {
          setIsProcessing(false)
        }
      } catch (err) {
        console.error('Auth init failed', err)
        // Let the error state below handle the UI
        setIsProcessing(false)
      }
    }

    handleAuth()
  }, [accessToken, setSession, authorize, exchangeToken])

  // Error State
  if (authorize.isError) {
    return (
      <div className="flex h-screen flex-col items-center justify-center p-4">
        <Alert variant="destructive" className="max-w-md">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Authentication Error</AlertTitle>
          <AlertDescription>
            {authorize.error?.message || 'Failed to initialize authentication.'}
          </AlertDescription>
        </Alert>
        <Button variant="outline" className="mt-4" onClick={() => window.location.reload()}>
          Retry
        </Button>
      </div>
    )
  }

  // Loading State
  if (isProcessing) {
    return <div className="flex h-screen items-center justify-center">Authenticating...</div>
  }

  // Authenticated State
  if (accessToken) {
    // CRITICAL FIX: If logged in but on /login page, redirect to Dashboard
    if (location.pathname === '/login') {
      return <Navigate to="/" replace />
    }
    return <>{children}</>
  }

  // Login Page Logic
  // If the backend challenge told us to go to /login, allow rendering it
  if (location.pathname === '/login') {
    return <>{children}</>
  }

  // Redirect Logic
  // If we aren't authenticated and aren't on /login, go there
  return <Navigate to="/login" replace />
}
