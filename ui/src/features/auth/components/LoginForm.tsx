import { useEffect, useState } from 'react'
import type { HTMLAttributes } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, LogIn } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useLocation, useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { PasswordInput } from '@/components/password-input'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { useRefreshToken } from '@/features/auth/api/useRefreshToken'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema'
import { cn } from '@/shared/lib/utils'
import { FormInput } from '@/shared/ui/form-input.tsx'

import { authApi } from '../api/authApi'
import type { ExecutionResult } from '../model/types'

interface UserAuthFormProps extends HTMLAttributes<HTMLFormElement> {
  redirectTo?: string
}

export function LoginForm({ className, redirectTo, ...props }: UserAuthFormProps) {
  // 1. Hooks & Stores
  const { realm = 'master' } = useParams()
  const { t } = useTranslation('common')
  const location = useLocation()

  // Access the global session store setters
  const setSession = useSessionStore((state) => state.setSession)
  const refreshTokenMutation = useRefreshToken()

  // 2. Local State Machine
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<ExecutionResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 3. INIT: Start the flow on mount
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

  // 4. Form Setup
  const form = useForm<LoginSchema>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  })

  // 5. SUBMIT: Handle "Next Step"
  const onSubmit = async (values: LoginSchema) => {
    if (!sessionId) return
    setIsLoading(true)
    setGlobalError(null)

    try {
      // Send inputs to the execution handler
      const res = await authApi.submitStep(sessionId, values)

      setSessionId(res.session_id)
      setCurrentStep(res.execution) // Update the UI to the next state
    } catch (error: any) {
      setGlobalError(error.message || 'An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  // 6. SUCCESS HANDLER: Hydrate Session & Redirect
  useEffect(() => {
    const handleSuccess = async () => {
      if (currentStep?.type !== 'Success') return

      try {
        // A. Hydrate the Session Immediately
        // We have the cookie (HttpOnly), now we fetch the Access Token
        // so the UI updates instantly without a reload.
        const token = await refreshTokenMutation.mutateAsync()
        setSession(token)

        // B. Calculate Redirect Target
        // 1. React Router Location (HashRouter support)
        const searchParams = new URLSearchParams(location.search)
        const savedRedirect = searchParams.get('redirect')

        // 2. Backend Suggestion (OIDC or configured flow redirect)
        const backendUrl = currentStep.payload.redirect_url

        let targetPath = '/'

        if (backendUrl && backendUrl !== '/' && backendUrl.startsWith('http')) {
          // OIDC External Redirect
          window.location.href = backendUrl
          return
        } else if (savedRedirect) {
          targetPath = decodeURIComponent(savedRedirect)
        } else if (redirectTo) {
          targetPath = redirectTo
        }

        // C. Perform Navigation (HashRouter Safe)
        const safePath = targetPath.startsWith('/') ? targetPath : `/${targetPath}`

        // Because we updated the store (setSession), AuthGuard will allow this
        // navigation immediately without blocking or looping.
        window.location.href = `${window.location.origin}/#${safePath}`
      } catch (err) {
        console.error('Login successful, but session hydration failed:', err)
        setGlobalError('Login succeeded but session could not be established.')
      }
    }

    if (currentStep?.type === 'Success') {
      void handleSuccess()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentStep])

  // --- RENDER LOGIC ---

  // A. Loading / Success State
  if (isLoading && !currentStep) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  // B. Success Message (Visual Feedback while redirecting)
  if (currentStep?.type === 'Success') {
    return (
      <div className="space-y-2 text-center text-green-600">
        <h3 className="text-lg font-medium">Login Successful</h3>
        <p className="text-sm">Finalizing session...</p>
      </div>
    )
  }

  // C. Failure State (Flow ended with fatal error)
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

  // D. Challenge State (Render the Form)
  if (currentStep?.type === 'Challenge') {
    const { screen_id } = currentStep.payload

    // CASE 1: Username/Password Screen
    // Supports "FORM" (legacy) and "username_password_node" (new)
    if (screen_id === 'FORM' || screen_id.includes('password') || screen_id.includes('login')) {
      return (
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className={cn('grid gap-3', className)}
            {...props}
          >
            {/* Global Error (API or Validation) */}
            {globalError && <div className="text-destructive mb-2 text-sm">{globalError}</div>}

            <FormInput
              control={form.control}
              name="username"
              label={t('LOGIN_PAGE.FIELDS.EMAIL')}
              placeholder={t('LOGIN_PAGE.FIELDS.EMAIL_PLACEHOLDER')}
            />

            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem className="relative">
                  <FormLabel>{t('LOGIN_PAGE.FIELDS.PASSWORD')}</FormLabel>
                  <FormControl>
                    <PasswordInput
                      placeholder={t('LOGIN_PAGE.FIELDS.PASSWORD_PLACEHOLDER')}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                  <Link
                    to="/forgot-password"
                    className="text-muted-foreground absolute end-0 -top-0.5 text-sm font-medium hover:opacity-75"
                  >
                    {t('LOGIN_PAGE.FORGOT_PASSWORD_LINK')}
                  </Link>
                </FormItem>
              )}
            />

            <Button className="mt-2" disabled={isLoading}>
              {isLoading ? <Loader2 className="mr-2 animate-spin" /> : <LogIn className="mr-2" />}
              {t('LOGIN_PAGE.SIGN_IN_BTN')}
            </Button>
          </form>
        </Form>
      )
    }

    // CASE 2: Unknown Screen
    return (
      <div className="rounded bg-yellow-50 p-4 text-yellow-800">
        Unrecognized Flow Step: <strong>{screen_id}</strong>
      </div>
    )
  }

  return null
}
