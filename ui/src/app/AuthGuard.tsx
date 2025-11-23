import { useEffect, useRef, useState } from 'react'

import { AlertCircle } from 'lucide-react'
import { Navigate, useLocation } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { refreshAccessToken } from '@/features/auth/api/authApi'
import { useOidcAuth } from '@/features/auth/api/useOidcAuth'
import { PKCE_STORAGE_KEY } from '@/shared/config/oidc'
import { generateCodeChallenge, generateCodeVerifier } from '@/shared/lib/pkce'

export const AuthGuard = ({ children }: { children: React.ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const location = useLocation()
  const [isProcessing, setIsProcessing] = useState(true)
  const initRan = useRef(false)

  const { authorize, exchangeToken } = useOidcAuth()

  useEffect(() => {
    const handleAuth = async () => {
      // 1. If we are already logged in (in memory), stop.
      if (accessToken) {
        setIsProcessing(false)
        return
      }

      if (initRan.current) return
      initRan.current = true

      // 2. Check for Auth Code (Callback from OIDC)
      const searchParams = new URLSearchParams(window.location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
        if (verifier) {
          exchangeToken.mutate(
            { code: authCode, verifier },
            {
              onSuccess: (data) => {
                setSession(data.access_token)
                // Clean URL
                const newUrl = window.location.pathname + window.location.hash
                window.history.replaceState({}, document.title, newUrl)
                sessionStorage.removeItem(PKCE_STORAGE_KEY)
                setIsProcessing(false)
              },
              onError: (error) => {
                console.error('Token exchange failed', error)
                // If exchange fails, we will fall through to restoration/login below
                // by NOT setting isProcessing to false here immediately
              },
            },
          )
        }
        return
      }

      // --- 3. SILENT REFRESH (Restore Session) ---
      // Before forcing a new login, check if we have a valid cookie
      try {
        console.log('[AuthGuard] Attempting silent refresh...')
        const token = await refreshAccessToken()
        setSession(token)
        setIsProcessing(false)
        return // Stop here, we are logged in!
      } catch (e) {
        console.log('[AuthGuard] Silent refresh failed, starting OIDC flow.')
      }
      // -------------------------------------------

      // 4. Start OIDC Flow (If no session and no code)
      const verifier = generateCodeVerifier()
      const challenge = await generateCodeChallenge(verifier)
      sessionStorage.setItem(PKCE_STORAGE_KEY, verifier)

      authorize.mutate(challenge, {
        onSuccess: (response) => {
          if (response.status === 'challenge' && response.challenge_page) {
            // Backend says "User needs to login", let the UI render the login page
            setIsProcessing(false)
            return
          }
        },
        onError: () => setIsProcessing(false),
      })
    }

    handleAuth()
  }, [accessToken, setSession, authorize, exchangeToken])

  // --- RENDER STATES ---

  // 1. Error State
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

  // 2. Loading State
  if (isProcessing || authorize.isPending || exchangeToken.isPending) {
    return <div className="flex h-screen items-center justify-center">Authenticating...</div>
  }

  // 3. Authenticated State
  if (accessToken) {
    // CRITICAL FIX: If logged in but on /login page, redirect to Dashboard
    if (location.pathname === '/login') {
      return <Navigate to="/" replace />
    }
    return <>{children}</>
  }

  // 4. Login Page Logic
  // If the backend challenge told us to go to /login, allow rendering it
  if (location.pathname === '/login') {
    return <>{children}</>
  }

  // 5. Redirect Logic
  // If we aren't authenticated and aren't on /login, go there
  return <Navigate to="/login" replace />
}
