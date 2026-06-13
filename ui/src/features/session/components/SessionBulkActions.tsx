import { type Table } from '@tanstack/react-table'
import { Trash2 } from 'lucide-react'

import type { Session } from '@/entities/session/model/types'
import { useRevokeSessions } from '@/features/session/api/useSessions'
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

interface SessionBulkActionsProps {
  table: Table<Session>
  currentSessionId: string | undefined
}

export function SessionBulkActions({ table, currentSessionId }: SessionBulkActionsProps) {
  const revokeSessions = useRevokeSessions()

  const selected = table.getFilteredSelectedRowModel().rows.map((row) => row.original)
  // The caller's current session is excluded server-side too, but reflect it in the count.
  const ids = selected.map((s) => s.id).filter((id) => id !== currentSessionId)

  if (ids.length === 0) return null

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="destructive" size="sm" className="h-8">
          <Trash2 className="mr-2 h-4 w-4" />
          Revoke {ids.length} session{ids.length === 1 ? '' : 's'}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent overlayClassName="bg-background/80 dot-grid text-muted-foreground/20">
        <AlertDialogHeader>
          <AlertDialogTitle>Revoke sessions</AlertDialogTitle>
          <AlertDialogDescription>
            Revoke {ids.length} selected session{ids.length === 1 ? '' : 's'}? Affected users will
            need to sign in again on those devices.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            onClick={async () => {
              await revokeSessions.mutateAsync(ids)
              table.toggleAllRowsSelected(false)
            }}
          >
            Revoke
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
