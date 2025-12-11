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
      // 1. If we already have a token in memory, we are good.
      if (accessToken) {
        setIsProcessing(false)
        return
      }

      // Prevent double-invocation in strict mode
      if (initRan.current) return
      initRan.current = true

      // 2. Check for OIDC Callback (Code in URL)
      // Note: In HashRouter, params might be in location.search OR window.location.search depending on setup.
      // We check both to be safe, prioritizing the router location.
      const searchParams = new URLSearchParams(location.search || window.location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
        if (verifier) {
          try {
            const data = await exchangeToken.mutateAsync({ code: authCode, verifier })
            setSession(data.access_token)
            sessionStorage.removeItem(PKCE_STORAGE_KEY)

            // Clean URL (remove code/state)
            const newUrl = window.location.pathname + window.location.hash.split('?')[0]
            window.history.replaceState({}, document.title, newUrl)
          } catch (err) {
            console.error('Token exchange failed', err)
          } finally {
            setIsProcessing(false)
          }
        } else {
          console.error('PKCE Verifier missing')
          setIsProcessing(false)
        }
        return
      }

      // 3. Silent Refresh (The most common path for returning users)
      try {
        // console.log('[AuthGuard] Attempting silent refresh...')
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
        return // Stop here if refresh worked
      } catch (e) {
        // console.log('[AuthGuard] Silent refresh failed, proceeding to login flow.')
      }

      // 4. If we are completely unauthenticated:
      // We generate PKCE challenge just in case we need to trigger an OIDC flow later,
      // but we DON'T block rendering. We allow the UI to decide (Login Page vs Redirect).
      const verifier = generateCodeVerifier()
      const challenge = await generateCodeChallenge(verifier)
      sessionStorage.setItem(PKCE_STORAGE_KEY, verifier)

      // Only trigger authorize if we are NOT on the login page already.
      // Triggers the OIDC /authorize call to get session ID cookies set up.
      try {
        const response = await authorize.mutateAsync(challenge)
        if (response.status === 'challenge') {
          // We are ready to show the login form
          setIsProcessing(false)
        }
      } catch (err) {
        console.error('Auth init failed', err)
        setIsProcessing(false)
      }
    }

    void handleAuth()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []) // Empty dependency array ensures this runs once on mount

  // --- RENDER LOGIC ---

  // A. Error State (Network failure on init)
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

  // B. Loading State
  if (isProcessing) {
    return <div className="flex h-screen items-center justify-center">Authenticating...</div>
  }

  // C. Authenticated State
  if (accessToken) {
    // If logged in but sitting on /login, move to the intended destination
    if (location.pathname === '/login') {
      const searchParams = new URLSearchParams(location.search)
      const redirect = searchParams.get('redirect')

      // If no redirect param, go to root.
      return <Navigate to={redirect || '/'} replace />
    }
    return <>{children}</>
  }

  // D. Unauthenticated - Login Page
  // Allow the login page to render itself so the user can see the form
  if (location.pathname === '/login') {
    return <>{children}</>
  }

  // E. Unauthenticated - Protected Route
  // Kick user to login, preserving the current location in the query param
  const currentPath = encodeURIComponent(location.pathname + location.search)
  return <Navigate to={`/login?redirect=${currentPath}`} replace />
}
