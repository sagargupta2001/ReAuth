import type { HTMLAttributes } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { Loader2, LogIn } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { PasswordInput } from '@/components/password-input'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema.ts'
import { cn } from '@/lib/utils.ts'

// This function handles the API call
async function executeLogin(credentials: LoginSchema) {
  const res = await fetch('/api/auth/login/execute', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ credentials }),
  })

  if (!res.ok) {
    const errorData = await res.json()
    throw new Error(errorData.error || 'Login failed')
  }

  return res.json()
}

interface UserAuthFormProps extends HTMLAttributes<HTMLFormElement> {
  redirectTo?: string
}

export function LoginForm({ className, redirectTo, ...props }: UserAuthFormProps) {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const setSession = useSessionStore((state) => state.setSession)

  const form = useForm<LoginSchema>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  })

  const mutation = useMutation({
    mutationFn: executeLogin,
    onSuccess: (data) => {
      // The backend response has two possibilities:
      if (data.status === 'challenge') {
        // e.g., MFA step - redirect to the next page in the flow
        navigate(data.nextUrl)
      } else if (data.status === 'redirect' && data.url) {
        // The flow is done. The backend generated an Auth Code.
        // We must redirect the browser to that URL (which is likely /?code=...)
        window.location.href = data.url
      } else if (data.access_token) {
        // SUCCESS! Set the session in our global store
        setSession(data.access_token)

        // Redirect to the page they were originally trying to access, or dashboard
        const redirectTo = searchParams.get('redirect') || '/'
        navigate(redirectTo, { replace: true })
      }
    },
  })

  const onSubmit = (values: LoginSchema) => {
    mutation.mutate(values)
  }

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className={cn('grid gap-3', className)}
        {...props}
      >
        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input placeholder="name@example.com" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem className="relative">
              <FormLabel>Password</FormLabel>
              <FormControl>
                <PasswordInput placeholder="********" {...field} />
              </FormControl>
              <FormMessage />
              <Link
                to="/forgot-password"
                className="text-muted-foreground absolute end-0 -top-0.5 text-sm font-medium hover:opacity-75"
              >
                Forgot password?
              </Link>
            </FormItem>
          )}
        />
        <Button className="mt-2" disabled={mutation.isPending}>
          {mutation.isPending ? <Loader2 className="animate-spin" /> : <LogIn />}
          Sign in
        </Button>
      </form>
    </Form>
  )
}
