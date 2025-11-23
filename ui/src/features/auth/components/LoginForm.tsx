import type { HTMLAttributes } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, LogIn } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { PasswordInput } from '@/components/password-input'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema'
import { cn } from '@/shared/lib/utils'

import { useLogin } from '../api/useLogin'

interface UserAuthFormProps extends HTMLAttributes<HTMLFormElement> {
  redirectTo?: string
}

export function LoginForm({ className, redirectTo, ...props }: UserAuthFormProps) {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const setSession = useSessionStore((state) => state.setSession)

  const loginMutation = useLogin()

  const form = useForm<LoginSchema>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  })

  const onSubmit = (values: LoginSchema) => {
    loginMutation.mutate(values, {
      onSuccess: (data) => {
        if (data.status === 'challenge' && data.challenge_page) navigate(data.challenge_page)
        else if (data.status === 'redirect' && data.url) window.location.href = data.url
        else if (data.access_token) {
          setSession(data.access_token)
          const target = searchParams.get('redirect') || '/'
          navigate(target, { replace: true })
        }
      },
      // Optional: Handle errors locally if needed, though React Query tracks it
      onError: (error) => {
        form.setError('root', { message: error.message })
      },
    })
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
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input placeholder="admin" {...field} />
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

        {/* Show global error if any */}
        {form.formState.errors.root && (
          <div className="text-destructive text-sm">{form.formState.errors.root.message}</div>
        )}

        <Button className="mt-2" disabled={loginMutation.isPending}>
          {loginMutation.isPending ? (
            <Loader2 className="mr-2 animate-spin" />
          ) : (
            <LogIn className="mr-2" />
          )}
          Sign in
        </Button>
      </form>
    </Form>
  )
}
