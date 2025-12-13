import { useEffect, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useLocation, useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'

import { authApi } from '../api/authApi'
import type { ExecutionResult } from '../model/types'
import { getScreenComponent } from './ScreenRegistry'

// This prevents double-execution even if the component unmounts/remounts
let initializationPromise: Promise<any> | null = null

export function AuthFlowExecutor() {
  const { realm = 'master' } = useParams()
  const location = useLocation()
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  const [sessionId, setSessionId] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<ExecutionResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 1. Singleton Initialization
  useEffect(() => {
    // If we already have data, don't run again
    if (sessionId) {
      setIsLoading(false)
      return
    }

    // Define the async worker
    const runInit = async () => {
      try {
        console.log('[Executor] Starting Flow Initialization...')
        const res = await authApi.startFlow(realm)
        console.log('[Executor] Success:', res.session_id)
        return res
      } catch (err) {
        console.error('[Executor] Failed:', err)
        throw err
      }
    }

    // If no promise exists, create one (The first and only one)
    if (!initializationPromise) {
      initializationPromise = runInit()
    }

    // Wait for the singleton promise
    let active = true
    initializationPromise
      .then((res) => {
        if (active) {
          setSessionId(res.session_id)
          setCurrentStep(res.execution)
          setIsLoading(false)
        }
      })
      .catch(() => {
        if (active) {
          setGlobalError('Failed to initialize login flow.')
          setIsLoading(false)
        }
        // Reset promise on error so we can retry later if needed
        initializationPromise = null
      })

    return () => {
      active = false
      // NOTE: We do NOT clear initializationPromise here.
      // We want the result to persist across remounts.
    }
  }, [realm, sessionId]) // Dependencies

  // 2. Submit Handler
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

  // 3. Success / Redirect Handler
  useEffect(() => {
    const handleSuccess = async () => {
      if (currentStep?.type !== 'Success') return

      try {
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)

        const searchParams = new URLSearchParams(location.search)
        const savedRedirect = searchParams.get('redirect')
        const backendUrl = currentStep.payload.redirect_url

        let targetPath = '/'
        if (backendUrl && backendUrl !== '/' && backendUrl.startsWith('http')) {
          window.location.href = backendUrl
          return
        } else if (savedRedirect) {
          targetPath = decodeURIComponent(savedRedirect)
        }

        const safePath = targetPath.startsWith('/') ? targetPath : `/${targetPath}`
        window.location.href = `${window.location.origin}/#${safePath}`
      } catch (err) {
        console.error('Session hydration failed:', err)
        setGlobalError('Login succeeded but session could not be established.')
      }
    }

    if (currentStep?.type === 'Success') {
      void handleSuccess()
    }
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
        <p className="text-sm">Finalizing session...</p>
      </div>
    )
  }

  if (currentStep?.type === 'Failure') {
    return (
      <div className="space-y-4">
        <div className="text-destructive rounded border border-red-100 bg-red-50 p-4 font-medium">
          Login Failed: {currentStep.payload.reason}
        </div>
        <Button
          variant="outline"
          className="w-full"
          onClick={() => {
            // Force full reload to reset the global singleton
            window.location.reload()
          }}
        >
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

    return (
      <div className="rounded bg-yellow-50 p-4 text-yellow-800">
        [Flow Error] Unrecognized Screen ID: <strong>{screen_id}</strong>
      </div>
    )
  }

  return null
}
