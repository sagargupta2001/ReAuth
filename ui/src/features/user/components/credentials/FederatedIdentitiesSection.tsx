import { Link2, MoreHorizontal } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import type { UserFederatedIdentity } from '@/features/user/api/useUserCredentials'
import { useUnlinkFederatedIdentity } from '@/features/user/api/useUserCredentials'
import { cn } from '@/lib/utils'

interface FederatedIdentitiesSectionProps {
  userId: string
  identities: UserFederatedIdentity[]
}

export function FederatedIdentitiesSection({ userId, identities }: FederatedIdentitiesSectionProps) {
  const unlinkFederatedIdentityMutation = useUnlinkFederatedIdentity(userId)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Federated identities</CardTitle>
      </CardHeader>
      <CardContent>
        {identities.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-10 text-center">
            <Link2 className="text-muted-foreground h-8 w-8" />
            <p className="text-muted-foreground text-sm">No federated identities linked.</p>
          </div>
        ) : (
          <div>
            {identities.map((identity, index) => {
              const isOnly = identities.length === 1
              const isFirst = index === 0
              const isLast = index === identities.length - 1

              return (
                <div
                  key={identity.id}
                  className={cn(
                    'bg-primary-foreground flex items-center justify-between gap-3 border p-3',
                    isOnly && 'rounded-2xl',
                    !isOnly && isFirst && 'rounded-t-2xl',
                    !isOnly && isLast && 'rounded-b-2xl',
                    !isLast && 'border-b-0',
                  )}
                >
                  <div className="flex min-w-0 items-center gap-3">
                    <Link2 className="text-muted-foreground h-4 w-4 shrink-0" />
                    <div className="min-w-0 space-y-1">
                      <div className="text-sm font-medium">{identity.provider_display_name}</div>
                      <div className="text-muted-foreground text-xs">
                        alias: {identity.provider_alias} | linked via: {identity.linked_via}
                      </div>
                      <div className="text-muted-foreground text-xs">
                        subject: {identity.subject}
                        {identity.external_email ? ` | email: ${identity.external_email}` : ''}
                      </div>
                      {identity.last_login_at ? (
                        <div className="text-muted-foreground text-xs">
                          last sign-in: {new Date(identity.last_login_at).toLocaleString()}
                        </div>
                      ) : null}
                    </div>
                  </div>

                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 shrink-0"
                        disabled={unlinkFederatedIdentityMutation.isPending}
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        variant="destructive"
                        onClick={() => {
                          if (
                            !window.confirm(
                              `Unlink ${identity.provider_display_name} from this user? This may block sign-in if no other credential remains.`,
                            )
                          ) {
                            return
                          }
                          unlinkFederatedIdentityMutation.mutate(identity.id)
                        }}
                      >
                        Unlink
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
