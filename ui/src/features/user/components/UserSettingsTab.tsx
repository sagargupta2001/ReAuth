import { useState } from 'react'

import { Ban, FileJson, LockKeyhole, Trash2 } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import type { User } from '@/entities/user/model/types'
import { useBanUser, useDeleteUser, useLockUser } from '@/features/user/api/useUserActions'
import { useUserCredentials } from '@/features/user/api/useUserCredentials'
import { UserJsonDialog } from '@/features/user/components/UserJsonDialog'
import { PasswordPolicySection } from '@/features/user/components/settings/PasswordPolicySection'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'
import { Skeleton } from '@/shared/ui/skeleton'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip'

type DangerAction = 'lock' | 'ban' | 'delete'

const dialogCopy: Record<
  DangerAction,
  { title: string; description: string; confirmText: string; destructive?: boolean }
> = {
  lock: {
    title: 'Lock User',
    description:
      'Are you sure you want to lock this user?\n\nThis will prevent the user from signing in to your application. The lockout duration can be modified in your realm settings. This action can be undone.',
    confirmText: 'Lock user',
  },
  ban: {
    title: 'Ban user',
    description:
      'Are you sure you want to ban this user?\n\nThis will prevent the user from signing in to your application for an indefinite period of time. This action can be undone.',
    confirmText: 'Ban user',
    destructive: true,
  },
  delete: {
    title: 'Delete development user',
    description:
      'Are you sure you want to delete this user?\n\nThis action is permanent and cannot be undone.',
    confirmText: 'Delete user',
    destructive: true,
  },
}

interface DangerSectionProps {
  icon: LucideIcon
  title: string
  description: string
  buttonLabel: string
  buttonVariant?: 'destructive' | 'outline'
  disabled?: boolean
  disabledTooltip?: string
  onClick: () => void
}

function DangerSection({
  icon: Icon,
  title,
  description,
  buttonLabel,
  buttonVariant = 'destructive',
  disabled,
  disabledTooltip,
  onClick,
}: DangerSectionProps) {
  const btn = (
    <Button
      type="button"
      variant={buttonVariant}
      className="gap-2"
      disabled={disabled}
      onClick={onClick}
    >
      <Icon className="h-4 w-4" />
      {buttonLabel}
    </Button>
  )

  return (
    <div className="border-destructive/50 bg-destructive/10 rounded-xl border p-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div className="flex items-start gap-3">
          <div className="bg-destructive/20 text-destructive rounded-full p-2">
            <Icon className="h-4 w-4" />
          </div>
          <div>
            <div className="text-destructive text-sm font-semibold">{title}</div>
            <p className="text-muted-foreground text-xs">{description}</p>
          </div>
        </div>
        {disabled && disabledTooltip ? (
          <TooltipProvider delayDuration={150}>
            <Tooltip>
              <TooltipTrigger asChild>
                <div>{btn}</div>
              </TooltipTrigger>
              <TooltipContent side="top" className="bg-popover text-popover-foreground border">
                {disabledTooltip}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        ) : (
          btn
        )}
      </div>
    </div>
  )
}

interface UserSettingsTabProps {
  userId: string
  user?: User
  onDeleted: () => void
}

export function UserSettingsTab({ userId, user, onDeleted }: UserSettingsTabProps) {
  const { data, isLoading } = useUserCredentials(userId)
  const [action, setAction] = useState<DangerAction | null>(null)
  const [jsonOpen, setJsonOpen] = useState(false)
  const currentUserId = useSessionStore((state) => state.user?.sub)
  const lockUser = useLockUser(userId)
  const banUser = useBanUser(userId)
  const deleteUser = useDeleteUser(userId)
  const isSelf = currentUserId === userId
  const isActionPending = lockUser.isPending || banUser.isPending || deleteUser.isPending
  const activeCopy = action ? dialogCopy[action] : null
  const selfTooltip = 'You cannot lock, ban, or delete your own account.'

  const handleConfirm = () => {
    if (action === 'lock') {
      lockUser.mutate(undefined, { onSuccess: () => setAction(null) })
      return
    }
    if (action === 'ban') {
      banUser.mutate(undefined, { onSuccess: () => setAction(null) })
      return
    }
    if (action === 'delete') {
      deleteUser.mutate(undefined, {
        onSuccess: () => {
          setAction(null)
          onDeleted()
        },
      })
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-20" />
        <Skeleton className="h-20" />
      </div>
    )
  }

  return (
    <div className="flex max-w-4xl flex-col gap-6">
      <PasswordPolicySection userId={userId} password={data?.password} />

      <Card>
        <CardHeader>
          <CardTitle>User Data</CardTitle>
          <CardDescription>Inspect the raw JSON representation of this user.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-primary-foreground flex flex-wrap items-center justify-between gap-4 rounded-2xl p-4">
            <div>
              <p className="text-sm font-medium">Raw JSON</p>
              <p className="text-muted-foreground text-sm">
                View the complete user object as JSON.
              </p>
            </div>
            <Button type="button" variant="outline" onClick={() => setJsonOpen(true)}>
              <FileJson className="h-4 w-4" />
              Show JSON
            </Button>
          </div>
        </CardContent>
      </Card>

      <DangerSection
        icon={LockKeyhole}
        title="Lock User"
        description="Temporarily prevents sign-in. Lockout duration is set in realm settings. This action can be undone."
        buttonLabel="Lock User"
        buttonVariant="outline"
        disabled={isSelf || isActionPending}
        disabledTooltip={isSelf ? selfTooltip : undefined}
        onClick={() => setAction('lock')}
      />

      <DangerSection
        icon={Ban}
        title="Ban User"
        description="Prevents sign-in indefinitely. This action can be undone."
        buttonLabel="Ban User"
        disabled={isSelf || isActionPending}
        disabledTooltip={isSelf ? selfTooltip : undefined}
        onClick={() => setAction('ban')}
      />

      <DangerSection
        icon={Trash2}
        title="Delete User"
        description="Permanently removes this user and all associated data. This cannot be undone."
        buttonLabel="Delete User"
        disabled={isSelf || isActionPending}
        disabledTooltip={isSelf ? selfTooltip : undefined}
        onClick={() => setAction('delete')}
      />

      <UserJsonDialog user={user} open={jsonOpen} onOpenChange={setJsonOpen} />

      {activeCopy ? (
        <ConfirmDialog
          open={Boolean(action)}
          onOpenChange={(open) => {
            if (!open) setAction(null)
          }}
          title={activeCopy.title}
          desc={
            <div className="space-y-3">
              {activeCopy.description.split('\n\n').map((line) => (
                <p key={line}>{line}</p>
              ))}
            </div>
          }
          confirmText={activeCopy.confirmText}
          destructive={activeCopy.destructive}
          isLoading={isActionPending}
          handleConfirm={handleConfirm}
        />
      ) : null}
    </div>
  )
}
