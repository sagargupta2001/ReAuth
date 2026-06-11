import { type ReactNode, useState } from 'react'

import {
  Ban,
  ChevronDown,
  LockKeyhole,
  Trash2,
} from 'lucide-react'

import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { useSessionStore } from '@/entities/session/model/sessionStore'
import type { User } from '@/entities/user/model/types'
import { useBanUser, useDeleteUser, useLockUser } from '@/features/user/api/useUserActions'
import { cn } from '@/lib/utils'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip'

type UserAction = 'lock' | 'ban' | 'delete'

interface UserHeaderActionsProps {
  userId: string
  user?: User
  onDeleted: () => void
}

const dialogCopy: Record<
  UserAction,
  {
    title: string
    description: string
    confirmText: string
    destructive?: boolean
  }
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

export function UserHeaderActions({ userId, user, onDeleted }: UserHeaderActionsProps) {
  const [action, setAction] = useState<UserAction | null>(null)
  const currentUserId = useSessionStore((state) => state.user?.sub)
  const lockUser = useLockUser(userId)
  const banUser = useBanUser(userId)
  const deleteUser = useDeleteUser(userId)
  const activeCopy = action ? dialogCopy[action] : null
  const isLoading = lockUser.isPending || banUser.isPending || deleteUser.isPending
  const isSelf = currentUserId === userId
  const disabledReason = isSelf ? 'You cannot lock, ban, or delete your own account.' : undefined

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

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="soft" disabled={!user || isLoading}>
            Actions
            <ChevronDown className="ml-2 h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <ActionMenuItem
            disabled={isSelf}
            disabledReason={disabledReason}
            onSelect={() => setAction('lock')}
          >
            <LockKeyhole className="h-4 w-4" />
            Lock user
          </ActionMenuItem>
          <ActionMenuItem
            disabled={isSelf}
            disabledReason={disabledReason}
            onSelect={() => setAction('ban')}
          >
            <Ban className="h-4 w-4" />
            Ban user
          </ActionMenuItem>
          <DropdownMenuSeparator />
          <ActionMenuItem
            variant="destructive"
            disabled={isSelf}
            disabledReason={disabledReason}
            onSelect={() => setAction('delete')}
          >
            <Trash2 className="h-4 w-4" />
            Delete user
          </ActionMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

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
          isLoading={isLoading}
          handleConfirm={handleConfirm}
        />
      ) : null}
    </>
  )
}

interface ActionMenuItemProps {
  children: ReactNode
  disabled?: boolean
  disabledReason?: string
  variant?: 'default' | 'destructive'
  onSelect: () => void
}

function ActionMenuItem({
  children,
  disabled = false,
  disabledReason,
  variant = 'default',
  onSelect,
}: ActionMenuItemProps) {
  const item = (
    <DropdownMenuItem
      variant={variant}
      aria-disabled={disabled}
      className={cn(
        disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer',
      )}
      onSelect={(event) => {
        if (disabled) {
          event.preventDefault()
          return
        }
        onSelect()
      }}
    >
      {children}
    </DropdownMenuItem>
  )

  if (!disabled || !disabledReason) return item

  return (
    <TooltipProvider delayDuration={150}>
      <Tooltip>
        <TooltipTrigger asChild>
          <div>{item}</div>
        </TooltipTrigger>
        <TooltipContent side="left" className="bg-popover text-popover-foreground border">
          {disabledReason}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
