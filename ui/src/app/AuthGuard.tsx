import { type ReactNode, useEffect, useRef, useState } from 'react'

import { AlertCircle } from 'lucide-react'
import { Navigate, useLocation, useNavigate } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useOidcAuth } from '@/features/auth/api/useOidcAuth'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken.ts'
import { PKCE_STORAGE_KEY } from '@/shared/config/oidc'

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
  if (window.location.pathname === '/login') {
    const search = window.location.search
    window.location.replace(`${window.location.origin}/#/login${search}`)
    return null
  }

  useEffect(() => {
    const handleAuth = async () => {
      // 1. If we already have a token, stop processing
      if (accessToken) {
        setIsProcessing(false)
        return
      }

      if (processingRef.current) return
      processingRef.current = true

      try {
        // --- 2. HANDLE OIDC CALLBACK (Code in URL) ---
        const searchParams = new URLSearchParams(window.location.search)
        const authCode = searchParams.get('code')

        if (authCode) {
          console.log('[AuthGuard] Detected OIDC Code. Exchanging...')
          const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)

          // Verify we initiated this request
          if (!verifier) {
            throw new Error('Missing PKCE verifier (this might be an old code).')
          }

          // Exchange Code for Token
          const data = await exchangeToken.mutateAsync({ code: authCode, verifier })
          setSession(data.access_token)
          sessionStorage.removeItem(PKCE_STORAGE_KEY)

          console.log('[AuthGuard] Token Exchanged. Restoring deep link...')

          // --- DEEP LINK RESTORATION ---
          const storedPath = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
          sessionStorage.removeItem(REDIRECT_STORAGE_KEY)

          let targetPath = storedPath || '/'
          if (!targetPath.startsWith('/')) targetPath = `/${targetPath}`

          // Clean URL (remove ?code=...)
          const cleanUrl = `${window.location.origin}/#${targetPath}`
          window.history.replaceState(null, '', cleanUrl)

          // Sync Router
          setTimeout(() => {
            navigate(targetPath, { replace: true })
            setIsProcessing(false)
          }, 50)

          return
        }

        // --- 3. SILENT REFRESH ---
        console.log('[AuthGuard] No code found, attempting silent refresh...')
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)
        setIsProcessing(false)
      } catch (err) {
        console.warn('[AuthGuard] Auth check failed:', err)

        // --- CRITICAL FIX: CLEAN URL ON FAILURE ---
        // If the code exchange failed (e.g. invalid code), strip it from the URL.
        // Otherwise, the app will keep trying to use the bad code on every render.
        const currentUrl = new URL(window.location.href)
        if (currentUrl.searchParams.has('code')) {
          const cleanUrl = `${window.location.origin}${window.location.hash}`
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
    if (location.pathname.includes('/login')) {
      const storedRedirect = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
      return <Navigate to={storedRedirect || '/'} replace />
    }
    return <>{children}</>
  }

  // --- UNAUTHENTICATED ---

  const isLoginPage = location.pathname === '/login' || location.pathname === '/login/'

  if (isLoginPage) {
    const searchParams = new URLSearchParams(location.search)
    const redirectIntent = searchParams.get('redirect')
    if (redirectIntent) {
      sessionStorage.setItem(REDIRECT_STORAGE_KEY, redirectIntent)
    }
    return <>{children}</>
  }

  const currentPath = encodeURIComponent(location.pathname + location.search)
  return <Navigate to={`/login?redirect=${currentPath}`} replace />
}
