import { useEffect, useMemo, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useLocation, useParams } from 'react-router-dom'

// <--- Import useLocation

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { authApi } from '@/features/auth/api/authApi.ts'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'
import { getScreenComponent } from '@/features/auth/components/ScreenRegistry.tsx'
import type { AuthExecutionResponse } from '@/entities/auth/model/types.ts'

// Global Singleton to prevent double-fetch in Strict Mode
let initializationPromise: Promise<AuthExecutionResponse> | null = null

export function AuthFlowExecutor() {
  const params = useParams()
  const location = useLocation() // <--- Hook to get ?client_id=...
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  // Determine Realm Priority:
  // 1. Query Param (?realm=tenant-a) <- Sent by OIDC Backend Redirect
  // 2. Route Param (/realms/tenant-a/login) <- Sent by direct link
  // 3. Default ('master')
  const realm = useMemo(() => {
    const searchParams = new URLSearchParams(location.search)
    return searchParams.get('realm') || params.realm || 'master'
  }, [location.search, params.realm])

  const [currentStep, setCurrentStep] = useState<AuthExecutionResponse | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 1. INITIALIZE FLOW (GET /api/auth/login)
  useEffect(() => {
    if (currentStep) {
      setIsLoading(false)
      return
    }

    const runInit = async () => {
      try {
        console.log(`[Executor] Starting Flow for realm: ${realm}`)

        // [FIX] We pass the extracted 'realm' variable here
        // verify your authApi.startFlow uses this first argument to build the URL:
        // `/api/realms/${realm}/auth/login${queryParams}`
        return await authApi.startFlow(realm, location.search)
      } catch (err) {
        console.error('[Executor] Init Failed:', err)
        throw err
      }
    }

    if (!initializationPromise) {
      initializationPromise = runInit()
    }

    let active = true
    initializationPromise
      .then((res) => {
        if (active) {
          setCurrentStep(res)
          setIsLoading(false)
        }
        initializationPromise = null
      })
      .catch((err) => {
        if (active) {
          setGlobalError('Failed to initialize login flow. ' + (err.message || ''))
          setIsLoading(false)
        }
        initializationPromise = null
      })

    return () => {
      active = false
    }
  }, [realm, location.search, currentStep])

  // 2. SUBMIT HANDLER
  const handleSubmit = async (data: Record<string, unknown>) => {
    setIsLoading(true)
    setGlobalError(null)

    try {
      // Pass realm here if your API needs it for the execution URL too
      // e.g. /api/realms/{realm}/auth/login/execute
      const res = await authApi.submitStep(realm, data)
      setCurrentStep(res)
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : 'An unexpected error occurred'
      setGlobalError(message)
    } finally {
      setIsLoading(false)
    }
  }

  // 3. SUCCESS / REDIRECT HANDLER
  useEffect(() => {
    const handleRedirect = async () => {
      if (currentStep?.status !== 'redirect') return

      try {
        const targetUrl = currentStep.url
        console.log('[Executor] Flow Complete. Redirecting to:', targetUrl)

        // Case A: Dashboard (Direct Login)
        if (targetUrl === '/') {
          const token = await refreshTokenMutation.mutateAsync()
          setSession(token)
          // Ensure we don't accidentally send query params to dashboard
          window.history.replaceState({}, document.title, '/')
          return
        }

        // Case B: OIDC External Redirect
        if (targetUrl.startsWith('http')) {
          window.location.href = targetUrl
          return
        }

        // Case C: Relative Redirect
        window.location.href = targetUrl
      } catch (err) {
        console.error('Session hydration failed:', err)
        setGlobalError('Login succeeded but session could not be established.')
      }
    }

    void handleRedirect()
  }, [currentStep, refreshTokenMutation, setSession])

  // --- RENDER ---
  if (isLoading && !currentStep) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  if (currentStep?.status === 'failure') {
    return (
      <div className="space-y-4">
        <div className="text-destructive rounded border border-red-100 bg-red-50 p-4 font-medium">
          Login Failed: {currentStep.message}
        </div>
        <Button className="w-full" onClick={() => window.location.reload()}>
          Try Again
        </Button>
      </div>
    )
  }

  if (currentStep?.status === 'redirect') {
    return (
      <div className="space-y-2 text-center text-green-600">
        <Loader2 className="mx-auto h-6 w-6 animate-spin" />
        <p className="text-sm">Redirecting...</p>
      </div>
    )
  }

  if (currentStep?.status === 'challenge') {
    const { challengeName, context } = currentStep
    const ScreenComponent = getScreenComponent(challengeName)

    if (ScreenComponent) {
      return (
        <ScreenComponent
          onSubmit={handleSubmit}
          isLoading={isLoading}
          error={globalError}
          context={context}
        />
      )
    }

    return <div className="p-4 text-red-500">Unknown Screen: {challengeName}</div>
  }

  return null
}
