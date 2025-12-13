import { type ReactNode, useEffect, useRef, useState } from 'react'

import { AlertCircle } from 'lucide-react'
import { Navigate, useLocation, useNavigate } from 'react-router-dom'

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
  const navigate = useNavigate()
  const [isProcessing, setIsProcessing] = useState(true)
  const initRan = useRef(false)

  const { authorize, exchangeToken } = useOidcAuth()
  const refreshTokenMutation = useRefreshToken()

  // --- 0. HASH ROUTER FIX ---
  // Ensure we are in Hash Mode. If backend sent us to /login, jump to /#/login
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
      // Check window.location because HashRouter ignores query params before the hash
      const searchParams = new URLSearchParams(window.location.search || location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        console.log('[AuthGuard] Detected OIDC Code. Exchanging...')
        const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
        if (verifier) {
          try {
            const data = await exchangeToken.mutateAsync({ code: authCode, verifier })
            setSession(data.access_token)
            sessionStorage.removeItem(PKCE_STORAGE_KEY)

            console.log('[AuthGuard] Token Exchanged. Cleaning URL...')

            // [FIX] CLEAN URL WITHOUT RELOAD (History API)
            // 1. Construct the destination URL (Base + Hash Target)
            let targetPath = sessionStorage.getItem(REDIRECT_STORAGE_KEY) || '/'
            sessionStorage.removeItem(REDIRECT_STORAGE_KEY)

            if (!targetPath.startsWith('/')) targetPath = `/${targetPath}`

            // 2. Use History API to change URL Bar *instantly* without reloading page
            // This removes "?code=..." and sets the correct hash path.
            const cleanUrl = `${window.location.origin}/#${targetPath}`
            window.history.replaceState(null, '', cleanUrl)

            // 3. Sync React Router internal state
            navigate(targetPath, { replace: true })
          } catch (err) {
            console.error('[AuthGuard] Token exchange failed', err)
            // If code is invalid/used, fall through to Silent Refresh logic
          } finally {
            setIsProcessing(false)
          }
        }
        // If we processed a code (success or fail), we return here to let the
        // next render cycle handle the 'Authenticated' or 'Unauthenticated' state.
        return
      }

      // 3. Silent Refresh
      try {
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
        return
      } catch (e) {
        // Refresh failed, proceed to login checks
      }

      // 4. Authentication Required Checks
      const isLoginPath = location.pathname.replace(/\/$/, '') === '/login'

      // CASE A: Login Page
      if (isLoginPath) {
        // Save intent if user typed /login?redirect=...
        const urlRedirect = new URLSearchParams(location.search).get('redirect')
        if (urlRedirect) {
          sessionStorage.setItem(REDIRECT_STORAGE_KEY, urlRedirect)
        }
        setIsProcessing(false)
        return
      }

      // CASE B: Protected Route -> Redirect to OIDC
      console.log('[AuthGuard] Redirecting to OIDC...')
      try {
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

  // --- AUTHENTICATED ---
  if (accessToken) {
    // If we are still sitting on /login (e.g. after silent refresh), move out
    if (location.pathname.replace(/\/$/, '') === '/login') {
      const storedRedirect = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
      const finalRedirect = storedRedirect || '/'

      sessionStorage.removeItem(REDIRECT_STORAGE_KEY)
      return <Navigate to={finalRedirect} replace />
    }

    return <>{children}</>
  }

  // --- UNAUTHENTICATED ---

  // Show Login Form
  if (location.pathname.replace(/\/$/, '') === '/login') {
    return <>{children}</>
  }

  // Force navigate to Login (breaks loops)
  const currentPath = encodeURIComponent(location.pathname + location.search)
  return <Navigate to={`/login?redirect=${currentPath}`} replace />
}
