import { useEffect, useMemo, useState } from 'react'
import type { FormEvent } from 'react'

import { useSearchParams } from 'react-router-dom'

import { Alert, AlertDescription, AlertTitle } from '@/components/alert'
import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { authApi } from '@/features/auth/api/authApi'
import { useAcceptInvitation } from '@/features/invitation/api/useInvitations'
import { Input } from '@/components/input'

export function InvitationAcceptPage() {
  const [searchParams] = useSearchParams()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [tokenStatus, setTokenStatus] = useState<'checking' | 'pending' | 'consumed' | 'expired'>(
    'checking',
  )

  const realm = useMemo(() => searchParams.get('realm')?.trim() ?? '', [searchParams])
  const token = useMemo(
    () =>
      searchParams.get('resume_token')?.trim() ?? searchParams.get('token')?.trim() ?? '',
    [searchParams],
  )

  const acceptMutation = useAcceptInvitation(realm)

  useEffect(() => {
    let cancelled = false

    if (!realm || !token) {
      setTokenStatus('expired')
      return () => {
        cancelled = true
      }
    }

    setTokenStatus('checking')
    void authApi
      .actionStatus(realm, token)
      .then((result) => {
        if (cancelled) return
        setTokenStatus(result.status)
      })
      .catch(() => {
        if (cancelled) return
        setTokenStatus('expired')
      })

    return () => {
      cancelled = true
    }
  }, [realm, token])

  const canSubmit =
    realm.length > 0 &&
    token.length > 0 &&
    tokenStatus === 'pending' &&
    username.trim().length >= 3 &&
    password.length >= 8 &&
    !acceptMutation.isPending

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault()
    if (!canSubmit) return
    setError(null)
    acceptMutation.mutate(
      {
        token,
        username: username.trim(),
        password,
      },
      {
        onSuccess: (response) => {
          window.location.assign(response.url)
        },
        onError: (mutationError) => {
          setError(
            mutationError instanceof Error
              ? mutationError.message
              : 'Failed to accept invitation.',
          )
        },
      },
    )
  }

  if (!realm || !token) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/30 px-6 py-10">
        <Card className="w-full max-w-lg">
          <CardHeader>
            <CardTitle>Invalid Invitation Link</CardTitle>
            <CardDescription>
              This invite link is missing required information. Please request a new invite.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  if (tokenStatus === 'checking') {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/30 px-6 py-10">
        <Card className="w-full max-w-lg">
          <CardHeader>
            <CardTitle>Checking Invitation</CardTitle>
            <CardDescription>Validating your invitation link...</CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  if (tokenStatus === 'expired' || tokenStatus === 'consumed') {
    return (
      <div className="flex min-h-screen items-center justify-center bg-muted/30 px-6 py-10">
        <Card className="w-full max-w-lg">
          <CardHeader>
            <CardTitle>Invitation Link Unavailable</CardTitle>
            <CardDescription>
              {tokenStatus === 'expired'
                ? 'This invitation link has expired. Ask your administrator to resend it.'
                : 'This invitation link has already been used. Ask your administrator to resend it if needed.'}
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 px-6 py-10">
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle>Accept Invitation</CardTitle>
          <CardDescription>Create your account credentials to continue.</CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <Alert variant="destructive" className="mb-4">
              <AlertTitle>Invitation failed</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <form className="space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="invitation-username">
                Username
              </label>
              <Input
                id="invitation-username"
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                placeholder="Choose a username"
                autoComplete="username"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="invitation-password">
                Password
              </label>
              <Input
                id="invitation-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder="At least 8 characters"
                autoComplete="new-password"
              />
            </div>
            <Button className="w-full" type="submit" disabled={!canSubmit}>
              {acceptMutation.isPending ? 'Accepting invitation...' : 'Accept invitation'}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
