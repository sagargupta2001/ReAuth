import { ShieldCheck } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Switch } from '@/components/switch'
import type { UserCredentials } from '@/features/user/api/useUserCredentials'
import { useUpdateUserPasswordPolicy } from '@/features/user/api/useUserCredentials'
import { cn } from '@/lib/utils'

interface PasswordPolicySectionProps {
  userId: string
  password?: UserCredentials['password']
}

const settingsRows = [
  {
    key: 'force_reset_on_next_login',
    title: 'Force password reset at next login',
    description:
      'User must set a new password after the next successful password authentication.',
  },
  {
    key: 'password_login_disabled',
    title: 'Disable password login for this user',
    description: 'Policy-gated: requires realm passkeys enabled and at least one enrolled passkey.',
  },
] as const

export function PasswordPolicySection({ userId, password }: PasswordPolicySectionProps) {
  const mutation = useUpdateUserPasswordPolicy(userId)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Password settings</CardTitle>
      </CardHeader>
      <CardContent>
        {settingsRows.map((row, index) => {
          const isFirst = index === 0
          const isLast = index === settingsRows.length - 1

          return (
            <div
              key={row.key}
              className={cn(
                'bg-primary-foreground flex items-center justify-between gap-3 border p-3',
                isFirst && 'rounded-t-2xl',
                isLast && 'rounded-b-2xl',
                !isLast && 'border-b-0',
              )}
            >
              <div className="flex min-w-0 items-start gap-3">
                <ShieldCheck className="text-muted-foreground mt-0.5 h-4 w-4 shrink-0" />
                <div className="min-w-0">
                  <div className="text-sm font-medium">{row.title}</div>
                  <div className="text-muted-foreground text-xs">{row.description}</div>
                </div>
              </div>
              <Switch
                checked={Boolean(password?.[row.key])}
                disabled={mutation.isPending}
                onCheckedChange={(checked) => {
                  mutation.mutate({ [row.key]: checked })
                }}
              />
            </div>
          )
        })}
      </CardContent>
    </Card>
  )
}
