import { type Table } from '@tanstack/react-table'
import { Trash2 } from 'lucide-react'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { type User } from '@/entities/user/model/types'
import { useDeleteUsers } from '@/features/user/api/useDeleteUsers.ts'
import { Button } from '@/components/button'
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
} from '@/components/alert-dialog'

interface UserBulkActionsProps {
  table: Table<User>
}

export function UserBulkActions({ table }: UserBulkActionsProps) {
  const realm = useActiveRealm()
  const deleteUsers = useDeleteUsers(realm)
  const selectedUsers = table.getFilteredSelectedRowModel().rows.map((row) => row.original)
  const deleteIds = selectedUsers.map((user) => user.id)

  if (deleteIds.length === 0) return null

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="destructive" size="sm" className="h-8">
          <Trash2 className="mr-2 h-4 w-4" />
          Delete {deleteIds.length} User{deleteIds.length !== 1 ? 's' : ''}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Users</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to permanently delete {deleteIds.length} user
            {deleteIds.length === 1 ? '' : 's'}? This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            onClick={async () => {
              await deleteUsers.mutateAsync(deleteIds)
              table.toggleAllRowsSelected(false)
            }}
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
