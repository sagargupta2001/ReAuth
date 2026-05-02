import { useState } from 'react'

import { KeyRound, ShieldCheck } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Input } from '@/components/input'
import { Switch } from '@/components/switch'
import {
  useRenameUserPasskey,
  useRevokeUserPasskey,
  useUpdateUserPassword,
  useUpdateUserPasswordPolicy,
  useUserCredentials,
} from '@/features/user/api/useUserCredentials'

interface UserCredentialsTabProps {
  userId: string
}

const maskCredentialId = (value: string) => {
  if (value.length <= 12) return value
  return `${value.slice(0, 6)}…${value.slice(-6)}`
}

export function UserCredentialsTab({ userId }: UserCredentialsTabProps) {
  const { data, isLoading } = useUserCredentials(userId)
  const updatePasswordMutation = useUpdateUserPassword(userId)
  const updatePasswordPolicyMutation = useUpdateUserPasswordPolicy(userId)
  const revokePasskeyMutation = useRevokeUserPasskey(userId)
  const renamePasskeyMutation = useRenameUserPasskey(userId)
  const [password, setPassword] = useState('')
  const [passkeyDraftNames, setPasskeyDraftNames] = useState<Record<string, string>>({})

  if (isLoading) return null

  const passkeys = data?.passkeys ?? []

  return (
    <div className="flex h-full w-full flex-col gap-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ShieldCheck className="h-4 w-4" />
            Password
          </CardTitle>
          <CardDescription>Update this user&apos;s password credential.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2">
            <Badge variant={data?.password.configured ? 'secondary' : 'outline'}>
              {data?.password.configured ? 'Configured' : 'Not Configured'}
            </Badge>
            <Badge variant={data?.password.password_login_disabled ? 'destructive' : 'outline'}>
              {data?.password.password_login_disabled ? 'Password Login Disabled' : 'Password Login Enabled'}
            </Badge>
          </div>
          <div className="space-y-3">
            <div className="flex items-center justify-between rounded-md border p-3">
              <div>
                <div className="text-sm font-medium">Force password reset at next login</div>
                <div className="text-muted-foreground text-xs">
                  User must set a new password after the next successful password authentication.
                </div>
              </div>
              <Switch
                checked={Boolean(data?.password.force_reset_on_next_login)}
                disabled={updatePasswordPolicyMutation.isPending}
                onCheckedChange={(checked) => {
                  updatePasswordPolicyMutation.mutate({ force_reset_on_next_login: checked })
                }}
              />
            </div>
            <div className="flex items-center justify-between rounded-md border p-3">
              <div>
                <div className="text-sm font-medium">Disable password login for this user</div>
                <div className="text-muted-foreground text-xs">
                  Policy-gated: requires realm passkeys enabled and at least one enrolled passkey.
                </div>
              </div>
              <Switch
                checked={Boolean(data?.password.password_login_disabled)}
                disabled={updatePasswordPolicyMutation.isPending}
                onCheckedChange={(checked) => {
                  updatePasswordPolicyMutation.mutate({ password_login_disabled: checked })
                }}
              />
            </div>
          </div>
          <div className="flex max-w-lg items-center gap-2">
            <Input
              type="password"
              placeholder="New password (8-100 chars)"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              disabled={updatePasswordMutation.isPending}
            />
            <Button
              onClick={() => {
                updatePasswordMutation.mutate(password, {
                  onSuccess: () => setPassword(''),
                })
              }}
              disabled={password.length < 8 || updatePasswordMutation.isPending}
            >
              Update Password
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <KeyRound className="h-4 w-4" />
            Passkeys
          </CardTitle>
          <CardDescription>Revoke enrolled passkeys for this user.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {passkeys.length === 0 ? (
            <div className="text-muted-foreground text-sm">No passkeys enrolled.</div>
          ) : (
            passkeys.map((credential) => (
              <div
                key={credential.id}
                className="flex items-center justify-between rounded-md border p-3"
              >
                <div className="space-y-1">
                  <div className="text-sm font-medium">
                    {credential.friendly_name || maskCredentialId(credential.credential_id_b64url)}
                  </div>
                  <div className="text-muted-foreground text-xs">
                    id: {maskCredentialId(credential.credential_id_b64url)} | sign count:{' '}
                    {credential.sign_count}
                  </div>
                  <div className="text-muted-foreground text-xs">
                    created: {new Date(credential.created_at).toLocaleString()}
                    {credential.last_used_at
                      ? ` | last used: ${new Date(credential.last_used_at).toLocaleString()}`
                      : ''}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Input
                    className="w-56"
                    placeholder="Friendly name"
                    value={
                      passkeyDraftNames[credential.id] ??
                      credential.friendly_name ??
                      ''
                    }
                    onChange={(event) => {
                      setPasskeyDraftNames((prev) => ({
                        ...prev,
                        [credential.id]: event.target.value,
                      }))
                    }}
                    disabled={renamePasskeyMutation.isPending}
                  />
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={renamePasskeyMutation.isPending}
                    onClick={() => {
                      const draft = passkeyDraftNames[credential.id]
                      const nextName =
                        typeof draft === 'string'
                          ? draft.trim() || null
                          : (credential.friendly_name ?? null)
                      renamePasskeyMutation.mutate({
                        credentialId: credential.id,
                        friendlyName: nextName,
                      })
                    }}
                  >
                    Save
                  </Button>
                  <Button
                    variant="destructive"
                    size="sm"
                    disabled={revokePasskeyMutation.isPending}
                    onClick={() => {
                      if (!window.confirm('Revoke this passkey? This action cannot be undone.')) {
                        return
                      }
                      revokePasskeyMutation.mutate(credential.id)
                    }}
                  >
                    Revoke
                  </Button>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  )
}
