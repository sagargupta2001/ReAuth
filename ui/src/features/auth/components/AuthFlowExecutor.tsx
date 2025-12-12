import { useEffect, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useLocation, useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'

import { authApi } from '../api/authApi'
import type { ExecutionResult } from '../model/types'
import { getScreenComponent } from './ScreenRegistry'

export function AuthFlowExecutor() {
  const { realm = 'master' } = useParams()
  const location = useLocation()
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  const [sessionId, setSessionId] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<ExecutionResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // INIT: Start Flow
  useEffect(() => {
    let active = true
    const init = async () => {
      try {
        const res = await authApi.startFlow(realm)
        if (active) {
          setSessionId(res.session_id)
          setCurrentStep(res.execution)
        }
      } catch (err) {
        if (active) setGlobalError('Failed to initialize login flow.')
      } finally {
        if (active) setIsLoading(false)
      }
    }
    void init()
    return () => {
      active = false
    }
  }, [realm])

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

  useEffect(() => {
    const handleSuccess = async () => {
      if (currentStep?.type !== 'Success') return

      try {
        // Hydrate Session
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)

        // Calculate Redirect
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

        // Hash Navigation
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentStep])

  if (isLoading && !currentStep)
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )

  // Success
  if (currentStep?.type === 'Success')
    return (
      <div className="space-y-2 text-center text-green-600">
        <h3 className="text-lg font-medium">Login Successful</h3>
        <p className="text-sm">Finalizing session...</p>
      </div>
    )

  // Failure
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

  // Challenge (The Dynamic Part)
  if (currentStep?.type === 'Challenge') {
    const { screen_id, context } = currentStep.payload

    // 1. Look up the component in registry
    const ScreenComponent = getScreenComponent(screen_id)

    // 2. Render it if found
    if (ScreenComponent)
      return (
        <ScreenComponent
          onSubmit={handleSubmit}
          isLoading={isLoading}
          error={globalError}
          context={context}
        />
      )

    // 3. Fallback for unknown screens
    return (
      <div className="rounded bg-yellow-50 p-4 text-yellow-800">
        [Flow Error] Unrecognized Screen ID: <strong>{screen_id}</strong>
      </div>
    )
  }

  return null
}
