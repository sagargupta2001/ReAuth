import { useSearchParams } from 'react-router-dom'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { LoginForm } from '@/features/auth/components/LoginForm.tsx'

export function LoginPage() {
  const [searchParams] = useSearchParams()
  const redirect = searchParams.get('redirect') ?? undefined

  return (
    <Card className="gap-4">
      <CardHeader>
        <CardTitle className="text-lg tracking-tight">Sign in</CardTitle>
        <CardDescription>
          Enter your email and password below to <br />
          log into your account
        </CardDescription>
      </CardHeader>
      <CardContent>
        <LoginForm redirectTo={redirect} />
      </CardContent>
    </Card>
  )
}
