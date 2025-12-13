import { useEffect, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'

import { authApi } from '../api/authApi'
import type { ExecutionResult } from '../model/types'
import { getScreenComponent } from './ScreenRegistry'

// Global Singleton to prevent double-fetch in Strict Mode
let initializationPromise: Promise<any> | null = null

export function AuthFlowExecutor() {
  const { realm = 'master' } = useParams()
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  const [sessionId, setSessionId] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<ExecutionResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 1. INITIALIZE FLOW
  useEffect(() => {
    // If state is already populated, stop.
    if (sessionId) {
      setIsLoading(false)
      return
    }

    const runInit = async () => {
      try {
        console.log('[Executor] Calling startFlow API...')
        const res = await authApi.startFlow(realm)
        console.log('[Executor] API Success. Session:', res.session_id)
        return res
      } catch (err) {
        console.error('[Executor] API Failed:', err)
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
          setSessionId(res.session_id)
          setCurrentStep(res.execution)
          setIsLoading(false)
        }
      })
      .catch((err) => {
        if (active) {
          // If the backend error is specifically "Session is closed",
          // our previous Backend fix should have handled it.
          // If we see it here, it's a fallback.
          setGlobalError('Failed to initialize login flow. ' + (err.message || ''))
          setIsLoading(false)
        }
        initializationPromise = null
      })

    return () => {
      active = false
    }
  }, [realm, sessionId])

  // 2. SUBMIT HANDLER
  const handleSubmit = async (data: any) => {
    if (!sessionId) return
    setIsLoading(true)
    setGlobalError(null)

    try {
      const res = await authApi.submitStep(sessionId, data)
      setSessionId(res.session_id)
      setCurrentStep(res.execution)
    } catch (error: any) {
      setGlobalError(error.message || 'An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  // 3. SUCCESS HANDLER
  useEffect(() => {
    const handleSuccess = async () => {
      if (currentStep?.type !== 'Success') return

      try {
        const token = await refreshTokenMutation.mutateAsync()

        // External Redirect Check (e.g. Google)
        const backendUrl = currentStep.payload.redirect_url
        if (backendUrl && backendUrl !== '/' && backendUrl.startsWith('http')) {
          window.location.href = backendUrl
          return
        }

        // Internal Success - Update Store
        // AuthGuard will detect this change and redirect to the saved URL.
        setSession(token)
      } catch (err) {
        console.error('Session hydration failed:', err)
        setGlobalError('Login succeeded but session could not be established.')
      }
    }

    if (currentStep?.type === 'Success') {
      void handleSuccess()
    }
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

  if (currentStep?.type === 'Success') {
    return (
      <div className="space-y-2 text-center text-green-600">
        <h3 className="text-lg font-medium">Login Successful</h3>
        <p className="text-sm">Redirecting...</p>
      </div>
    )
  }

  if (currentStep?.type === 'Failure') {
    return (
      <div className="space-y-4">
        <div className="text-destructive rounded border border-red-100 bg-red-50 p-4 font-medium">
          Login Failed: {currentStep.payload.reason}
        </div>
        <Button variant="outline" className="w-full" onClick={() => window.location.reload()}>
          Try Again
        </Button>
      </div>
    )
  }

  if (currentStep?.type === 'Challenge') {
    const { screen_id, context } = currentStep.payload
    const ScreenComponent = getScreenComponent(screen_id)

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

    return <div className="p-4 text-red-500">Unknown Screen: {screen_id}</div>
  }

  return null
}
