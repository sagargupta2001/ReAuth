import { useEffect, useState } from 'react'

import { useNavigate } from 'react-router-dom'

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
import { apiClient } from '@/shared/api/client'

type SetupStatus = {
  required: boolean
}

export function SetupPage() {
  const navigate = useNavigate()
  const [statusChecked, setStatusChecked] = useState(false)
  const [token, setToken] = useState('')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let active = true
    apiClient
      .get<SetupStatus>('/api/system/setup/status')
      .then((data) => {
        if (!active) return
        if (!data.required) {
          navigate('/', { replace: true })
          return
        }
        setStatusChecked(true)
      })
      .catch((err) => {
        if (!active) return
        setError(err instanceof Error ? err.message : 'Failed to check setup status.')
      })
      .finally(() => {
        if (active) setStatusChecked(true)
      })

    return () => {
      active = false
    }
  }, [navigate])

  const canSubmit =
    token.trim().length > 0 && username.trim().length > 0 && password.trim().length > 0

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (!canSubmit || isSubmitting) return
    setIsSubmitting(true)
    setError(null)
    const trimmedToken = token.trim()
    const trimmedUsername = username.trim()
    try {
      await apiClient.post('/api/system/setup', {
        token: trimmedToken,
        username: trimmedUsername,
        password,
      })
      navigate('/login', { replace: true })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Setup failed.')
    } finally {
      setIsSubmitting(false)
    }
  }

  if (!statusChecked) {
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
