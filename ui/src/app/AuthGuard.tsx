import { useEffect, useRef, useState } from 'react'

import { Navigate, useLocation } from 'react-router-dom'

import { useSessionStore } from '@/entities/session/model/sessionStore'
import { refreshAccessToken } from '@/features/auth/api/authApi'
import { oidcApi } from '@/features/auth/api/oidc'
// <-- 1. Import this
import { PKCE_STORAGE_KEY } from '@/shared/config/oidc'
import { generateCodeChallenge, generateCodeVerifier } from '@/shared/lib/pkce'

export const AuthGuard = ({ children }: { children: React.ReactNode }) => {
  const { accessToken, setSession } = useSessionStore()
  const location = useLocation()
  const [isProcessing, setIsProcessing] = useState(true)
  const initRan = useRef(false)

  useEffect(() => {
    const handleAuth = async () => {
      // 1. If we are already logged in (in memory), stop.
      if (accessToken) {
        setIsProcessing(false)
        return
      }

      if (initRan.current) return
      initRan.current = true

      // 2. Check for OIDC Callback (Auth Code)
      const searchParams = new URLSearchParams(window.location.search)
      const authCode = searchParams.get('code')

      if (authCode) {
        // ... (Your existing code exchange logic is correct) ...
        console.log('[AuthGuard] Auth code found. Exchanging for token...')
        try {
          const verifier = sessionStorage.getItem(PKCE_STORAGE_KEY)
          if (!verifier) throw new Error('Missing PKCE verifier')

          const data = await oidcApi.exchangeToken(authCode, verifier)
          setSession(data.access_token)

          const newUrl = window.location.pathname + window.location.hash
          window.history.replaceState({}, document.title, newUrl)
          sessionStorage.removeItem(PKCE_STORAGE_KEY)
        } catch (err) {
          console.error('[AuthGuard] Token exchange failed:', err)
        } finally {
          setIsProcessing(false)
        }
        return
      }

      // --- 3. THIS IS THE FIX: Try Silent Refresh ---
      // Before we force them to login, let's see if they have a valid cookie.
      try {
        console.log('[AuthGuard] Attempting silent refresh via cookie...')
        const token = await refreshAccessToken() // Calls /api/auth/refresh
        setSession(token) // Restore the session in memory
        setIsProcessing(false)
        return // Stop here, we are logged in!
      } catch (err) {
        console.log('[AuthGuard] Silent refresh failed (user truly logged out).')
        // If this fails, we proceed to Step 4 (OIDC Flow)
      }
      // ---------------------------------------------

      // 4. No code, no cookie? Start the OIDC Flow.
      console.log('[AuthGuard] No session. Starting OIDC flow...')
      try {
        const verifier = generateCodeVerifier()
        const challenge = await generateCodeChallenge(verifier)
        sessionStorage.setItem(PKCE_STORAGE_KEY, verifier)

        const response = await oidcApi.authorize(challenge)

        if (response.status === 'challenge' && response.challenge_page) {
          setIsProcessing(false)
          return
        }
      } catch (err) {
        console.error('[AuthGuard] Auth initialization failed:', err)
        setIsProcessing(false)
      }
    }

    handleAuth()
  }, [accessToken, setSession])

  // --- RENDER STATES ---

  if (isProcessing) {
    return <div className="flex h-screen items-center justify-center">Authenticating...</div>
  }

  if (accessToken) {
    return <>{children}</>
  }

  if (location.pathname === '/login') {
    return <>{children}</>
  }

  return <Navigate to="/login" replace />
}
