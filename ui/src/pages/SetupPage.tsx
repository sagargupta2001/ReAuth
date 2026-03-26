import { useEffect, useState } from 'react'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/card'
import { Input } from '@/components/input'
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
    if (setupStatus.isError) {
      setError(
        setupStatus.error instanceof Error
          ? setupStatus.error.message
          : 'Failed to check setup status.',
      )
    }
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

  if (setupStatus.isLoading) {
    return <div className="flex h-screen items-center justify-center">Checking setup...</div>
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 px-6 py-10">
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Initialize ReAuth</CardTitle>
          <CardDescription>
            Enter the setup token from the server console to create the first master admin.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <Alert variant="destructive" className="mb-4">
              <AlertTitle>Setup failed</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <form className="space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="setup-token">
                Setup token
              </label>
              <Input
                id="setup-token"
                value={token}
                onChange={(event) => setToken(event.target.value)}
                placeholder="Paste the setup token"
                autoComplete="off"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="setup-username">
                Admin username
              </label>
              <Input
                id="setup-username"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                placeholder="admin"
                autoComplete="username"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="setup-password">
                Admin password
              </label>
              <Input
                id="setup-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder="Create a strong password"
                autoComplete="new-password"
              />
            </div>
            <Button className="w-full" type="submit" disabled={!canSubmit || isSubmitting}>
              {isSubmitting ? 'Creating admin...' : 'Create master admin'}
            </Button>
          </form>
        </CardContent>
        <CardFooter className="text-xs text-muted-foreground">
          Setup is available only until the first master admin is created.
        </CardFooter>
      </Card>
    </div>
  )
}
