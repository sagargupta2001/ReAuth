import { useEffect, useMemo, useRef, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useLocation, useNavigate, useParams } from 'react-router-dom'

// <--- Import useLocation

import { Button } from '@/components/button'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { authApi } from '@/features/auth/api/authApi.ts'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'
import { getScreenComponent } from '@/features/auth/components/ScreenRegistry.tsx'
import type { AuthExecutionResponse } from '@/entities/auth/model/types.ts'
import { REDIRECT_STORAGE_KEY } from '@/shared/config/redirect'

// Global Singleton to prevent double-fetch in Strict Mode
let initializationPromise: Promise<AuthExecutionResponse> | null = null

export function AuthFlowExecutor() {
  return <BaseAuthFlowExecutor flowPath="login" />
}

type BaseAuthFlowExecutorProps = {
  flowPath?: 'login' | 'register' | 'reset'
}

export function BaseAuthFlowExecutor({ flowPath = 'login' }: BaseAuthFlowExecutorProps) {
  const params = useParams()
  const location = useLocation() // <--- Hook to get ?client_id=...
  const navigate = useNavigate()
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

  const resumeToken = useMemo(() => {
    const searchParams = new URLSearchParams(location.search)
    return searchParams.get('resume_token') || searchParams.get('action_token')
  }, [location.search])

  const clientId = useMemo(() => {
    const searchParams = new URLSearchParams(location.search)
    return searchParams.get('client_id') || undefined
  }, [location.search])

  const redirectParam = useMemo(() => {
    const searchParams = new URLSearchParams(location.search)
    return searchParams.get('redirect')
  }, [location.search])

  const [currentStep, setCurrentStep] = useState<AuthExecutionResponse | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)
  const redirectHandledRef = useRef(false)

  useEffect(() => {
    if (!redirectParam) return
    sessionStorage.setItem(REDIRECT_STORAGE_KEY, redirectParam)
  }, [redirectParam])

  // 1. INITIALIZE FLOW (GET /api/auth/login)
  useEffect(() => {
    if (currentStep) {
      setIsLoading(false)
      return
    }

    const runInit = async () => {
      try {
        console.log(`[Executor] Starting Flow for realm: ${realm}`)

        if (resumeToken) {
          const response = await authApi.resumeFlow(realm, resumeToken)
          const cleaned = new URLSearchParams(location.search)
          cleaned.delete('resume_token')
          cleaned.delete('action_token')
          const search = cleaned.toString()
          const nextUrl = search ? `${location.pathname}?${search}` : location.pathname
          window.history.replaceState({}, document.title, nextUrl)
          return response
        }

        // [FIX] We pass the extracted 'realm' variable here
        // verify your authApi.startFlow uses this first argument to build the URL:
        // `/api/realms/${realm}/auth/login${queryParams}`
        return await authApi.startFlow(realm, flowPath, location.search)
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
          setGlobalError('Failed to initialize auth flow. ' + (err.message || ''))
          setIsLoading(false)
        }
        initializationPromise = null
      })

    return () => {
      active = false
    }
  }, [realm, location.pathname, location.search, currentStep, resumeToken, flowPath])

  // 2. SUBMIT HANDLER
  const handleSubmit = async (data: Record<string, unknown>) => {
    setIsLoading(true)
    setGlobalError(null)

    try {
      // Pass realm here if your API needs it for the execution URL too
      // e.g. /api/realms/{realm}/auth/login/execute
      const res = await authApi.submitStep(realm, flowPath, data)
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
    if (currentStep?.status !== 'redirect') {
      redirectHandledRef.current = false
    }
    const handleRedirect = async () => {
      if (currentStep?.status !== 'redirect') return
      if (redirectHandledRef.current) return
      redirectHandledRef.current = true

      try {
        const targetUrl = currentStep.url
        console.log('[Executor] Flow Complete. Redirecting to:', targetUrl)

        // Case A: Dashboard (Direct Login)
        if (targetUrl === '/') {
          const token = await refreshTokenMutation.mutateAsync()
          setSession(token)
          const storedRedirect = sessionStorage.getItem(REDIRECT_STORAGE_KEY)
          sessionStorage.removeItem(REDIRECT_STORAGE_KEY)
          if (storedRedirect && storedRedirect.startsWith('/') && !storedRedirect.startsWith('//')) {
            navigate(storedRedirect, { replace: true })
          } else {
            navigate('/', { replace: true })
          }
          return
        }

        // Case B: OIDC External Redirect
        if (targetUrl.startsWith('http')) {
          window.location.href = targetUrl
          return
        }

        // Case C: Relative Redirect
        if (targetUrl.startsWith('/')) {
          navigate(targetUrl, { replace: true })
          if (targetUrl.startsWith('/login')) {
            setCurrentStep(null)
            setIsLoading(false)
            redirectHandledRef.current = false
          }
          return
        }
        window.location.href = targetUrl
      } catch (err) {
        console.error('Session hydration failed:', err)
        setGlobalError('Login succeeded but session could not be established.')
        setCurrentStep({
          status: 'failure',
          message: 'Login succeeded but session could not be established.',
        })
      }
    }

    void handleRedirect()
  }, [currentStep, refreshTokenMutation, setSession, navigate])

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
          Flow Failed: {currentStep.message}
        </div>
        <Button className="w-full" onClick={() => window.location.reload()}>
          Try Again
        </Button>
      </div>
    )
  }

  if (currentStep?.status === 'redirect') {
    return (
      <div className="flex min-h-[40vh] flex-col items-center justify-center space-y-2 text-center text-green-600">
        <Loader2 className="h-6 w-6 animate-spin" />
        <p className="text-sm">Redirecting...</p>
      </div>
    )
  }

  if (currentStep?.status === 'awaiting_action') {
    const { challengeName, context } = currentStep
    const ScreenComponent = getScreenComponent(challengeName)

    if (ScreenComponent) {
      return (
        <ScreenComponent
          onSubmit={handleSubmit}
          isLoading={isLoading}
          error={globalError}
          context={context}
          realm={realm}
          clientId={clientId}
        />
      )
    }

    return (
      <div className="space-y-2 text-center text-muted-foreground">
        <Loader2 className="mx-auto h-6 w-6 animate-spin" />
        <p className="text-sm">Waiting for verification...</p>
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
          realm={realm}
          clientId={clientId}
        />
      )
    }

    return <div className="p-4 text-red-500">Unknown Screen: {challengeName}</div>
  }

  return null
}
