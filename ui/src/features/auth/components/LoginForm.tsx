import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { useNavigate, useSearchParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema.ts'

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

export function LoginForm() {
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
    <Card className="w-[350px]">
      <CardHeader>
        <CardTitle>Sign In</CardTitle>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
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
                <FormItem>
                  <FormLabel>Password</FormLabel>
                  <FormControl>
                    <Input type="password" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            {mutation.isError && (
              <p className="text-destructive text-sm font-medium">{mutation.error.message}</p>
            )}
            <Button type="submit" className="w-full" disabled={mutation.isPending}>
              {mutation.isPending ? 'Signing In...' : 'Sign In'}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  )
}
