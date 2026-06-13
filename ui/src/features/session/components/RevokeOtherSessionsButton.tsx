import { ShieldX } from 'lucide-react'

import { useRevokeOtherSessions } from '@/features/session/api/useSessions'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/shared/ui/alert-dialog.tsx'
import { Button } from '@/shared/ui/button.tsx'

export function RevokeOtherSessionsButton() {
  const revokeOthers = useRevokeOtherSessions()

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button size="sm" className="h-9">
          <ShieldX className="h-4 w-4" />
          Revoke all other sessions
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent overlayClassName="bg-background/80 dot-grid text-muted-foreground/20">
        <AlertDialogHeader>
          <AlertDialogTitle>Revoke all other sessions</AlertDialogTitle>
          <AlertDialogDescription>
            This signs you out of every session except the one you are currently using. Your current
            session is never affected.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            onClick={() => revokeOthers.mutate()}
          >
            Revoke all others
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
