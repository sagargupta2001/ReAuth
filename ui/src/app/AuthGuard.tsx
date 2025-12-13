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

  // --- 0. HASH ROUTER FIX ---
  // If the backend redirected to "/login" (server path) instead of "/#/login",
  // HashRouter sees this as "/" (root). We must manually correct this.
  if (window.location.pathname === '/login') {
    // Preserve query params (e.g. ?redirect=...) and move them before the hash if needed,
    // or keep them as is. For HashRouter, usually /#/login?params is best.
    const search = window.location.search
    window.location.replace(`${window.location.origin}/#/login${search}`)
    return null
  }
  // --------------------------

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
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
        return
      } catch (e) {
        // console.log('[AuthGuard] Silent refresh failed, proceeding to login flow.')
      }

      // 4. Authentication Required

      // CASE A: We are explicitly on the login page (via HashRouter)
      // Normalize check to handle trailing slashes
      const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

      if (isLoginPath) {
        setIsProcessing(false)
        return
      }

      // CASE B: Protected Route -> Redirect to Backend
      try {
        const verifier = generateCodeVerifier()
        const challenge = await generateCodeChallenge(verifier)
        sessionStorage.setItem(PKCE_STORAGE_KEY, verifier)

        await authorize.mutateAsync(challenge)
      } catch (err) {
        console.error('Auth redirect failed', err)
        setIsProcessing(false)
      }
    }

    void handleAuth()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // --- RENDER LOGIC ---

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

  if (isProcessing) {
    return <div className="flex h-screen items-center justify-center">Authenticating...</div>
  }

  // Authenticated
  if (accessToken) {
    const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

    if (isLoginPath) {
      const searchParams = new URLSearchParams(location.search)
      const redirect = searchParams.get('redirect')
      return <Navigate to={redirect || '/'} replace />
    }
    return <>{children}</>
  }

  // Allow Login Page to render
  if (location.pathname.replace(/\/$/, '') === '/login') {
    return <>{children}</>
  }

  // Fallback (e.g. waiting for redirect to kick in)
  return <div className="flex h-screen items-center justify-center">Redirecting...</div>
}
