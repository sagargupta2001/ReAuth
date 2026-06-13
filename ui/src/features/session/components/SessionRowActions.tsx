import { useState } from 'react'

import { LogOut, MoreHorizontal, ShieldAlert, Trash2, UserX } from 'lucide-react'

import type { Session } from '@/entities/session/model/types'
import {
  useRevokeSession,
  useRevokeUserSessions,
  useStepUpSession,
} from '@/features/session/api/useSessions'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/shared/ui/alert-dialog.tsx'
import { Button } from '@/shared/ui/button.tsx'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu.tsx'

type ConfirmKind = 'terminate' | 'step_up' | 'revoke_user' | null

interface SessionRowActionsProps {
  session: Session
  currentSessionId: string | undefined
  onViewDetails: (session: Session) => void
}

export function SessionRowActions({
  session,
  currentSessionId,
  onViewDetails,
}: SessionRowActionsProps) {
  const [confirm, setConfirm] = useState<ConfirmKind>(null)

  const revokeSession = useRevokeSession()
  const stepUpSession = useStepUpSession()
  const revokeUserSessions = useRevokeUserSessions()

  const isCurrent = session.id === currentSessionId

  const dialogs: Record<
    Exclude<ConfirmKind, null>,
    { title: string; description: string; action: string; run: () => void }
  > = {
    terminate: {
      title: 'Terminate session',
      description:
        'This immediately revokes this session. The user will need to sign in again on that device.',
      action: 'Terminate',
      run: () => revokeSession.mutate(session.id),
    },
    step_up: {
      title: 'Force re-authentication',
      description:
        'The session keeps working until its access token expires, then must re-authenticate (and pass any MFA your login flow requires) before it can refresh again.',
      action: 'Require re-auth',
      run: () => stepUpSession.mutate(session.id),
    },
    revoke_user: {
      title: "Revoke user's account sessions",
      description:
        'This revokes every active session for this user across the realm, signing them out everywhere. Requires user-write permission.',
      action: 'Revoke all',
      run: () => revokeUserSessions.mutate(session.user_id),
    },
  }

  const active = confirm ? dialogs[confirm] : null

  return (
    <div className="flex justify-end" onClick={(e) => e.stopPropagation()}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8" aria-label="Session actions">
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuItem onSelect={() => onViewDetails(session)}>
            <ShieldAlert className="mr-2 h-4 w-4" />
            View details / token JSON
          </DropdownMenuItem>
          <DropdownMenuItem
            disabled={isCurrent}
            onSelect={() => setConfirm('step_up')}
            title={isCurrent ? 'Cannot step-up your current session' : undefined}
          >
            <LogOut className="mr-2 h-4 w-4" />
            Force re-authentication
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            disabled={isCurrent}
            onSelect={() => setConfirm('terminate')}
            className="text-destructive focus:text-destructive"
            title={isCurrent ? 'Cannot revoke your current session here' : undefined}
          >
            <Trash2 className="mr-2 h-4 w-4" />
            Terminate session
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={() => setConfirm('revoke_user')}
            className="text-destructive focus:text-destructive"
          >
            <UserX className="mr-2 h-4 w-4" />
            Revoke user&apos;s account sessions
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{active?.title}</AlertDialogTitle>
            <AlertDialogDescription>{active?.description}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={() => {
                active?.run()
                setConfirm(null)
              }}
            >
              {active?.action}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
