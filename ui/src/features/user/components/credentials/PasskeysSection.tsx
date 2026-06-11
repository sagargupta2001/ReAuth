import { useState } from 'react'

import { KeyRound, MoreHorizontal } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Input } from '@/components/input'
import type { UserPasskeyCredential } from '@/features/user/api/useUserCredentials'
import {
  useRenameUserPasskey,
  useRevokeUserPasskey,
} from '@/features/user/api/useUserCredentials'
import { cn } from '@/lib/utils'

interface PasskeysSectionProps {
  userId: string
  passkeys: UserPasskeyCredential[]
}

const maskCredentialId = (value: string) => {
  if (value.length <= 12) return value
  return `${value.slice(0, 6)}...${value.slice(-6)}`
}

export function PasskeysSection({ userId, passkeys }: PasskeysSectionProps) {
  const revokePasskeyMutation = useRevokeUserPasskey(userId)
  const renamePasskeyMutation = useRenameUserPasskey(userId)
  const [passkeyDraftNames, setPasskeyDraftNames] = useState<Record<string, string>>({})
  const busy = revokePasskeyMutation.isPending || renamePasskeyMutation.isPending

  return (
    <Card>
      <CardHeader>
        <CardTitle>Passkeys</CardTitle>
      </CardHeader>
      <CardContent>
        {passkeys.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-10 text-center">
            <KeyRound className="text-muted-foreground h-8 w-8" />
            <p className="text-muted-foreground text-sm">No passkeys enrolled.</p>
          </div>
        ) : (
          <div>
            {passkeys.map((credential, index) => {
              const isOnly = passkeys.length === 1
              const isFirst = index === 0
              const isLast = index === passkeys.length - 1
              const friendlyName =
                passkeyDraftNames[credential.id] ?? credential.friendly_name ?? ''
              const displayName =
                credential.friendly_name || maskCredentialId(credential.credential_id_b64url)

              return (
                <div
                  key={credential.id}
                  className={cn(
                    'bg-primary-foreground flex items-center justify-between gap-3 border p-3',
                    isOnly && 'rounded-2xl',
                    !isOnly && isFirst && 'rounded-t-2xl',
                    !isOnly && isLast && 'rounded-b-2xl',
                    !isLast && 'border-b-0',
                  )}
                >
                  <div className="flex min-w-0 items-center gap-3">
                    <KeyRound className="text-muted-foreground h-4 w-4 shrink-0" />
                    <div className="min-w-0 space-y-1">
                      <div className="text-sm font-medium">{displayName}</div>
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
                  </div>

                  <div className="flex shrink-0 items-center gap-2">
                    <Input
                      className="hidden w-48 lg:block"
                      placeholder="Friendly name"
                      value={friendlyName}
                      onChange={(event) =>
                        setPasskeyDraftNames((prev) => ({
                          ...prev,
                          [credential.id]: event.target.value,
                        }))
                      }
                      disabled={renamePasskeyMutation.isPending}
                    />
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon" className="h-8 w-8" disabled={busy}>
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem
                          onClick={() => {
                            renamePasskeyMutation.mutate({
                              credentialId: credential.id,
                              friendlyName: friendlyName.trim() || null,
                            })
                          }}
                        >
                          Save friendly name
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                          variant="destructive"
                          onClick={() => {
                            if (
                              !window.confirm(
                                'Revoke this passkey? This action cannot be undone.',
                              )
                            ) {
                              return
                            }
                            revokePasskeyMutation.mutate(credential.id)
                          }}
                        >
                          Revoke
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
