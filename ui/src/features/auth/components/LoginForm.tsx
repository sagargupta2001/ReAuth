import { useEffect, useState } from 'react'
import type { HTMLAttributes } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, LogIn } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link, useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { PasswordInput } from '@/components/password-input'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema'
import { cn } from '@/shared/lib/utils'
import { FormInput } from '@/shared/ui/form-input.tsx'

import { authApi } from '../api/authApi'
import type { ExecutionResult } from '../model/types'

interface UserAuthFormProps extends HTMLAttributes<HTMLFormElement> {
  redirectTo?: string
}

export function LoginForm({ className, redirectTo, ...props }: UserAuthFormProps) {
  // If your router has /realms/:realm/login, get it. Otherwise default to 'default'
  const { realm = 'master' } = useParams()
  const { t } = useTranslation('common')

  // -- STATE MACHINE --
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [currentStep, setCurrentStep] = useState<ExecutionResult | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [globalError, setGlobalError] = useState<string | null>(null)

  // 1. INIT: Start the flow on mount
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
        setGlobalError('Failed to initialize login flow.')
      } finally {
        if (active) setIsLoading(false)
      }
    }
    void init()
    return () => {
      active = false
    }
  }, [realm])

  // Form Setup
  const form = useForm<LoginSchema>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  })

  // 2. SUBMIT: Handle "Next Step"
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
      // If the API returns a standard error (not a flow failure)
      setGlobalError(error.message || 'An unexpected error occurred')
    } finally {
      setIsLoading(false)
    }
  }

  if (isLoading && !currentStep) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  if (currentStep?.type === 'Success') {
    setTimeout(() => {
      const searchParams = new URLSearchParams(window.location.search) // Note: useLocation().search is better if available, but window works for reload logic
      const savedRedirect = searchParams.get('redirect')
      const backendUrl = currentStep.payload.redirect_url

      let targetPath = '/'

      // 1. Determine Target
      if (backendUrl && backendUrl !== '/' && backendUrl.startsWith('http')) {
        window.location.href = backendUrl
        return
      } else if (savedRedirect) {
        targetPath = decodeURIComponent(savedRedirect)
      } else if (redirectTo) {
        targetPath = redirectTo
      }

      // 2. Format for HashRouter
      const safePath = targetPath.startsWith('/') ? targetPath : `/${targetPath}`
      const newUrl = `${window.location.origin}/#${safePath}`

      // 3. FORCE RELOAD (The Fix)
      // We set the location, then explicitly reload to reset AuthGuard state
      if (window.location.href === newUrl) {
        // We are already on the right URL but state is stale
        window.location.reload()
      } else {
        // Navigate and reload
        window.location.assign(newUrl)
        // Small timeout to ensure the browser registers the URL change before reloading
        setTimeout(() => window.location.reload(), 100)
      }
    }, 500)

    return (
      <div className="space-y-2 text-center text-green-600">
        <h3 className="text-lg font-medium">Login Successful</h3>
        <p className="text-sm">Loading application...</p>
      </div>
    )
  }

  // C. Failure State (Flow ended with error)
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
  // We check if the screen_id matches what this component knows how to render.
  if (currentStep?.type === 'Challenge') {
    const { screen_id } = currentStep.payload

    // CASE 1: Username/Password Screen
    // You might name this "username_password_node" or similar in Rust
    if (screen_id === 'FORM' || screen_id.includes('password') || screen_id.includes('login')) {
      return (
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className={cn('grid gap-3', className)}
            {...props}
          >
            {/* Show error from previous attempt if it was a "Failure" step that we recovered from or API error */}
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

    // CASE 2: Unknown Screen (Future proofing)
    return (
      <div className="rounded bg-yellow-50 p-4 text-yellow-800">
        Unrecognized Flow Step: <strong>{screen_id}</strong>
      </div>
    )
  }

  return null
}
