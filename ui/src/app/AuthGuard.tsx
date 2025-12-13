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

const REDIRECT_STORAGE_KEY = 'reauth_post_login_redirect'

export const AuthGuard = ({ children }: { children: ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const location = useLocation()
  const [isProcessing, setIsProcessing] = useState(true)
  const initRan = useRef(false)

  const { authorize, exchangeToken } = useOidcAuth()
  const refreshTokenMutation = useRefreshToken()

  // --- 0. HASH ROUTER PATCH ---
  // Fixes the mismatch where backend redirects to /login but app lives at /#/login
  if (window.location.pathname === '/login') {
    const search = window.location.search
    window.location.replace(`${window.location.origin}/#/login${search}`)
    return null
  }

  useEffect(() => {
    const handleAuth = async () => {
      // 1. If we have a token, we are done.
      if (accessToken) {
        setIsProcessing(false)
        return
      }

      if (initRan.current) return
      initRan.current = true

      // 2. Handle OIDC Callback (Code in URL)
      const searchParams = new URLSearchParams(location.search || window.location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        console.log('[AuthGuard] Detected OIDC Code. Exchanging...')
        const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
        if (verifier) {
          try {
            const data = await exchangeToken.mutateAsync({ code: authCode, verifier })
            setSession(data.access_token)
            sessionStorage.removeItem(PKCE_STORAGE_KEY)

            // Clean URL but do NOT redirect yet.
            // The "Authenticated" block below will handle navigation.
            const newUrl = window.location.pathname + window.location.hash.split('?')[0]
            window.history.replaceState({}, document.title, newUrl)
          } catch (err) {
            console.error('[AuthGuard] Token exchange failed', err)
          } finally {
            setIsProcessing(false)
          }
        }
        return
      }

      // 3. Silent Refresh
      try {
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
        return
      } catch (e) {
        // Refresh failed, proceed to checks
      }

      // 4. CHECK: Are we already on the Login Page?
      const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

      if (isLoginPath) {
        console.log('[AuthGuard] On Login Page. Rendering children.')

        // Save 'redirect' param if present (e.g. /login?redirect=/admin)
        const urlRedirect = searchParams.get('redirect')
        if (urlRedirect) {
          sessionStorage.setItem(REDIRECT_STORAGE_KEY, urlRedirect)
        }

        // STOP HERE. Do NOT trigger authorize.
        setIsProcessing(false)
        return
      }

      // 5. Protected Route -> Trigger OIDC Flow
      console.log('[AuthGuard] Unauthenticated on protected route. Redirecting to OIDC...')
      try {
        // Save where the user wanted to go
        const returnUrl = location.pathname + location.search
        sessionStorage.setItem(REDIRECT_STORAGE_KEY, returnUrl)

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

  // --- AUTHENTICATED STATE ---
  if (accessToken) {
    const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

    // If logged in, but on /login, move them to the saved destination
    if (isLoginPath) {
      const storedRedirect = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
      const finalRedirect = storedRedirect || '/'

      console.log('[AuthGuard] Logged in. Redirecting to:', finalRedirect)
      if (storedRedirect) sessionStorage.removeItem(REDIRECT_STORAGE_KEY)

      return <Navigate to={finalRedirect} replace />
    }

    return <>{children}</>
  }

  // --- UNAUTHENTICATED STATE ---

  const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

  // If on /login, RENDER THE FORM
  if (isLoginPath) {
    return <>{children}</>
  }

  // If on protected route, force navigation to /login
  // This helps break any loops by putting us into the "isLoginPath" bucket
  const currentPath = encodeURIComponent(location.pathname + location.search)
  return <Navigate to={`/login?redirect=${currentPath}`} replace />
}
