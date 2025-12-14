import { useEffect, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'

import { authApi } from '../api/authApi'
import type { AuthExecutionResponse } from '../model/types'
import { getScreenComponent } from './ScreenRegistry'

// Global Singleton to prevent double-fetch in Strict Mode
let initializationPromise: Promise<any> | null = null

export function AuthFlowExecutor() {
  const { realm = 'master' } = useParams()
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  // We store the raw API response here
  const [currentStep, setCurrentStep] = useState<AuthExecutionResponse | null>(null)

  // Track internal loading state
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 1. INITIALIZE FLOW (GET /api/auth/login)
  useEffect(() => {
    // If state is already populated, stop.
    if (currentStep) {
      setIsLoading(false)
      return
    }

    const runInit = async () => {
      try {
        console.log('[Executor] Starting Flow for realm:', realm)
        return await authApi.startFlow(realm)
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
          // res is the AuthExecutionResponse JSON directly
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
  }, [realm, currentStep])

  // 2. SUBMIT HANDLER (POST /api/auth/login/execute)
  const handleSubmit = async (data: any) => {
    setIsLoading(true)
    setGlobalError(null)

    try {
      // The backend now accepts a generic JSON payload.
      // For PasswordNode, we expect { username, password } inside data.
      const res = await authApi.submitStep(data)
      setCurrentStep(res)
    } catch (error: any) {
      // API errors (500s) or network errors
      setGlobalError(error.message || 'An unexpected error occurred')
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

        // Case A: Redirect to Dashboard (Root)
        if (targetUrl === '/') {
          // Hydrate the session via cookie -> token exchange
          const token = await refreshTokenMutation.mutateAsync()
          setSession(token)
          // The AppRouter will handle rendering the dashboard now that session is set
          return
        }

        // Case B: External Redirect (OIDC Callback, Google, etc.)
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

    handleRedirect()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentStep])

  // --- RENDER ---

  if (isLoading && !currentStep) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  // STATUS: FAILURE (Flow Terminated)
  if (currentStep?.status === 'failure') {
    return (
      <div className="space-y-4">
        <div className="text-destructive rounded border border-red-100 bg-red-50 p-4 font-medium">
          Login Failed: {currentStep.message}
        </div>
        <Button variant="outline" className="w-full" onClick={() => window.location.reload()}>
          Try Again
        </Button>
      </div>
    )
  }

  // STATUS: REDIRECT (Showing spinner while redirecting)
  if (currentStep?.status === 'redirect') {
    return (
      <div className="space-y-2 text-center text-green-600">
        <Loader2 className="mx-auto h-6 w-6 animate-spin" />
        <p className="text-sm">Redirecting...</p>
      </div>
    )
  }

  // STATUS: CHALLENGE (Render UI)
  if (currentStep?.status === 'challenge') {
    const { challengeName, context } = currentStep
    const ScreenComponent = getScreenComponent(challengeName) // e.g. "login-password"

    if (ScreenComponent) {
      return (
        <ScreenComponent
          onSubmit={handleSubmit}
          isLoading={isLoading}
          error={globalError} // Network/System errors
          context={context} // Business/Validation errors (e.g. "Invalid Password")
        />
      )
    }

    return <div className="p-4 text-red-500">Unknown Screen: {challengeName}</div>
  }

  return null
}
