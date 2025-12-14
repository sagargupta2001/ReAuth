import { type ReactNode, useEffect, useRef, useState } from 'react'

import { AlertCircle } from 'lucide-react'
import { Navigate, useLocation, useNavigate } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useOidcAuth } from '@/features/auth/api/useOidcAuth'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken.ts'
import { PKCE_STORAGE_KEY } from '@/shared/config/oidc'

// Ensure this key matches exactly what you use elsewhere
const REDIRECT_STORAGE_KEY = 'reauth_post_login_redirect'

export const AuthGuard = ({ children }: { children: ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const location = useLocation()
  const navigate = useNavigate()
  const [isProcessing, setIsProcessing] = useState(true)

  // Ref to prevent double-firing in React 18 Strict Mode
  const processingRef = useRef(false)

  const { authorize, exchangeToken } = useOidcAuth()
  const refreshTokenMutation = useRefreshToken()

  // --- 0. HASH ROUTER FIX (PRE-RENDER) ---
  // If backend sent us to /login (root path), jump to /#/login
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

      if (processingRef.current) return
      processingRef.current = true

      try {
        // --- 2. HANDLE OIDC CALLBACK (Code in URL) ---
        // Look at window.location.search explicitly (ignoring HashRouter)
        const searchParams = new URLSearchParams(window.location.search)
        const authCode = searchParams.get('code')

        if (authCode) {
          console.log('[AuthGuard] Detected OIDC Code. Verifying...')
          const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)

          // --- FIX 1: Handle Missing Verifier Gracefully ---
          if (!verifier) {
            console.warn('[AuthGuard] Missing PKCE verifier. Code is stale or invalid.')
            // Do NOT throw. Just clean URL and force user to login again.
            const cleanUrl = `${window.location.origin}/#/login`
            window.history.replaceState(null, '', cleanUrl)
            window.location.href = cleanUrl // Hard redirect to break loop
            return
          }

          // Exchange Code for Token
          const data = await exchangeToken.mutateAsync({ code: authCode, verifier })
          setSession(data.access_token)

          // Cleanup Security Keys
          sessionStorage.removeItem(PKCE_STORAGE_KEY)

          // --- DEEP LINK RESTORATION ---
          const storedPath = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
          console.log('[AuthGuard] Restoring Path from Storage:', storedPath)

          sessionStorage.removeItem(REDIRECT_STORAGE_KEY)

          // Normalize Target Path
          let targetPath = storedPath || '/'
          if (!targetPath.startsWith('/')) targetPath = `/${targetPath}`

          // --- FIX 2: Aggressive URL Cleaning ---
          // 1. Clean the browser URL bar first (removes ?code=...)
          const cleanUrl = `${window.location.origin}/#${targetPath}`
          window.history.replaceState(null, '', cleanUrl)

          console.log('[AuthGuard] Navigating to:', targetPath)

          // 2. Wait a tick, then let React Router handle the view change
          setTimeout(() => {
            navigate(targetPath, { replace: true })
            setIsProcessing(false)
          }, 100)

          return
        }

        // --- 3. SILENT REFRESH ---
        console.log('[AuthGuard] No code found, checking refresh token...')
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
      } catch (err) {
        console.warn('[AuthGuard] Auth check failed:', err)

        // --- FIX 3: Safety Cleanup on Error ---
        // If anything failed, strip the code so we don't loop.
        const currentUrl = new URL(window.location.href)
        if (currentUrl.searchParams.has('code')) {
          console.log('[AuthGuard] Cleaning broken code from URL')
          const cleanUrl = `${window.location.origin}/#/login`
          window.history.replaceState(null, '', cleanUrl)
        }
        setIsProcessing(false)
      } finally {
        processingRef.current = false
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
    // If we are stuck on /login after success, bump to home or stored path
    if (location.pathname.includes('/login')) {
      const storedRedirect = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
      return <Navigate to={storedRedirect || '/'} replace />
    }
    return <>{children}</>
  }

  // --- UNAUTHENTICATED ---

  const isLoginPage = location.pathname === '/login' || location.pathname === '/login/'

  if (isLoginPage) {
    // Save intent if user landed on /#/login?redirect=...
    const searchParams = new URLSearchParams(location.search)
    const redirectIntent = searchParams.get('redirect')

    if (redirectIntent) {
      console.log('[AuthGuard] Saving redirect intent:', redirectIntent)
      sessionStorage.setItem(REDIRECT_STORAGE_KEY, redirectIntent)
    }
    return <>{children}</>
  }

  // Redirect to Login Page
  const currentPath = encodeURIComponent(location.pathname + location.search)
  return <Navigate to={`/login?redirect=${currentPath}`} replace />
}
