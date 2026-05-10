import type { ColumnDef } from '@tanstack/react-table'
import { format } from 'date-fns'
import { MoreHorizontal } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import type { Invitation } from '@/entities/invitation/model/types'
import { DataTableColumnHeader } from '@/shared/ui/data-table'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu'

const invitationStatusVariant: Record<Invitation['status'], 'secondary' | 'success' | 'warning' | 'destructive'> = {
  pending: 'warning',
  accepted: 'success',
  expired: 'secondary',
  revoked: 'destructive',
}

const invitationStatusLabel: Record<Invitation['status'], string> = {
  pending: 'Pending',
  accepted: 'Accepted',
  expired: 'Expired',
  revoked: 'Revoked',
}

interface InvitationColumnsOptions {
  onResend: (invitationId: string) => void
  onRevoke: (invitationId: string) => void
  actionsDisabled?: boolean
}

export function getInvitationColumns(options: InvitationColumnsOptions): ColumnDef<Invitation>[] {
  const { onResend, onRevoke, actionsDisabled = false } = options

  return [
    {
      accessorKey: 'email',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Email" />,
      cell: ({ row }) => <span className="font-medium">{row.getValue('email')}</span>,
      enableSorting: true,
      size: 250
    },
    {
      accessorKey: 'status',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
      cell: ({ row }) => {
        const status = row.getValue('status') as Invitation['status']
        return <Badge variant={invitationStatusVariant[status]}>{invitationStatusLabel[status]}</Badge>
      },
      enableSorting: true,
    },
    {
      accessorKey: 'expires_at',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Expires" />,
      cell: ({ row }) => {
        const value = row.getValue('expires_at') as string
        return <span className="text-muted-foreground text-sm">{format(new Date(value), 'MMM d, yyyy, h:mm a')}</span>
      },
      enableSorting: true,
    },
    {
      accessorKey: 'last_sent_at',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Last Sent" />,
      cell: ({ row }) => {
        const value = row.getValue('last_sent_at') as string | null | undefined
        if (!value) return <span className="text-muted-foreground text-sm">Never</span>
        return <span className="text-muted-foreground text-sm">{format(new Date(value), 'MMM d, yyyy, h:mm a')}</span>
      },
      enableSorting: true,
    },
    {
      accessorKey: 'resend_count',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Resends" />,
      cell: ({ row }) => <span className="text-muted-foreground text-sm">{row.getValue('resend_count')}</span>,
      enableSorting: true,
    },
    {
      accessorKey: 'created_at',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Created" />,
      cell: ({ row }) => {
        const value = row.getValue('created_at') as string
        return <span className="text-muted-foreground text-sm">{format(new Date(value), 'MMM d, yyyy, h:mm a')}</span>
      },
      enableSorting: true,
    },
    {
      id: 'actions',
      cell: ({ row }) => {
        const invitation = row.original
        const canAct = invitation.status === 'pending' && !actionsDisabled

        return (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem disabled={!canAct} onClick={() => onResend(invitation.id)}>
                Resend invite
              </DropdownMenuItem>
              <DropdownMenuItem disabled={!canAct} onClick={() => onRevoke(invitation.id)}>
                Revoke invite
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )
      },
      enableSorting: false,
    },
  ]
}
