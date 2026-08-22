import { useEffect, useState } from 'react'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/card'
import { SETUP_SEALED_STORAGE_KEY } from '@/shared/config/setup'
import { useSetupBootstrap } from '@/features/setup/api/useSetupBootstrap'
import { useSetupStatus } from '@/features/setup/api/useSetupStatus'
import { useNavigate } from 'react-router-dom'

export function SetupPage() {
  const navigate = useNavigate()
  const [token, setToken] = useState('')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const setupStatus = useSetupStatus()
  const setupMutation = useSetupBootstrap()
  const isSubmitting = setupMutation.isPending

  useEffect(() => {
    if (setupStatus.isError)
      setError(
        setupStatus.error instanceof Error
          ? setupStatus.error.message
          : 'Failed to check setup status.',
      )
  }, [setupStatus.error, setupStatus.isError])

  useEffect(() => {
    if (setupStatus.isLoading) return
    if (!setupStatus.data?.required) {
      localStorage.setItem(SETUP_SEALED_STORAGE_KEY, '1')
      window.location.replace(`${window.location.origin}/#/login`)
      return
    }
    localStorage.removeItem(SETUP_SEALED_STORAGE_KEY)
  }, [setupStatus.data?.required, setupStatus.isLoading])

  const canSubmit =
    token.trim().length > 0 && username.trim().length > 0 && password.trim().length > 0

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (!canSubmit || setupMutation.isPending) return
    setError(null)
    const trimmedToken = token.trim()
    const trimmedUsername = username.trim()
    setupMutation.mutate(
      {
        token: trimmedToken,
        username: trimmedUsername,
        password,
      },
      {
        onSuccess: () => {
          navigate('/login', { replace: true })
          setTimeout(() => {
            window.location.replace(`${window.location.origin}/#/login`)
          }, 50)
        },
        onError: (err) => {
          setError(err instanceof Error ? err.message : 'Setup failed.')
        },
      },
    )
  }

  if (setupStatus.isLoading) 
    return <div className="flex h-screen items-center justify-center">Checking setup...</div>
  

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden px-6 py-10">
      <div aria-hidden="true" className="pulse-glow pointer-events-none absolute inset-0" />
      <form className="relative w-full max-w-lg" onSubmit={handleSubmit}>
        {/*
          Composed from the Card primitives rather than SectionCard: this is a
          centred hero card, not a settings card, and it deliberately has no
          inset content panel. CardContent is p-1 by default, so the padding
          below is doing real work — do not drop it.
        */}
        <Card>
          <CardHeader className="items-center text-center">
            <img rel="icon" src="/reauth.svg" alt="logo" className="mb-2 h-14 w-14" />
            <CardTitle>Initialize ReAuth</CardTitle>
            <CardDescription>
              Enter the setup token from the server console to create the first master admin.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 px-6 pb-6">
            {error && (
              <Alert variant="destructive">
                <AlertTitle>Setup failed</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-2">
              <Label htmlFor="setup-token">Setup token</Label>
              <Input
                id="setup-token"
                value={token}
                onChange={(event) => setToken(event.target.value)}
                placeholder="Paste the setup token"
                autoComplete="off"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="setup-username">Admin username</Label>
              <Input
                id="setup-username"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                placeholder="admin"
                autoComplete="username"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="setup-password">Admin password</Label>
              <Input
                id="setup-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder="Create a strong password"
                autoComplete="new-password"
              />
            </div>
            <Button className="mt-2 w-full" type="submit" disabled={!canSubmit || isSubmitting}>
              {isSubmitting ? 'Creating admin...' : 'Create master admin'}
            </Button>
          </CardContent>
          <CardFooter className="text-muted-foreground justify-center text-center text-xs">
            Setup is available only until the first master admin is created.
          </CardFooter>
        </Card>
      </form>
    </div>
  )
}
