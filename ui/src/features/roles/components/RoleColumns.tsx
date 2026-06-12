import type { ColumnDef } from '@tanstack/react-table'
import { format } from 'date-fns'
import { Shield } from 'lucide-react'

import type { Role } from '@/features/roles/api/useRoles.ts'
import { Badge } from '@/shared/ui/badge'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

export const roleColumns: ColumnDef<Role>[] = [
  {
    accessorKey: 'name',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Role" />,
    cell: ({ row }) => {
      const name = row.getValue('name') as string
      const description = row.original.description

      return (
        <div className="flex min-w-0 items-center gap-2">
          <Shield className="text-muted-foreground h-4 w-4 shrink-0" />
          <div className="flex min-w-0 flex-col">
            <span className="text-foreground truncate font-medium">{name}</span>
            <span className="text-muted-foreground truncate text-xs">
              {description || 'No description'}
            </span>
          </div>
        </div>
      )
    },
    enableSorting: true,
  },
  {
    id: 'scope',
    accessorKey: 'client_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Scope" />,
    cell: ({ row }) => {
      const isClientRole = Boolean(row.original.client_id)

      return (
        <Badge variant="outline" className="text-muted-foreground text-xs font-normal">
          {isClientRole ? 'Client' : 'Realm'}
        </Badge>
      )
    },
    enableSorting: false,
  },
  {
    accessorKey: 'user_count',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Users" />,
    cell: ({ row }) => {
      const count = row.original.user_count ?? 0
      return <span className="text-muted-foreground text-sm">{count}</span>
    },
    enableSorting: true,
    size: 100,
  },
  {
    accessorKey: 'permission_count',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Permissions" />,
    cell: ({ row }) => {
      const count = row.original.permission_count ?? 0
      return <span className="text-muted-foreground text-sm">{count}</span>
    },
    enableSorting: true,
    size: 120,
  },
  {
    accessorKey: 'created_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Created" />,
    cell: ({ row }) => {
      const value = row.original.created_at
      if (!value) return <span className="text-muted-foreground text-sm">-</span>
      return (
        <span className="text-muted-foreground text-sm">
          {format(new Date(value), 'MMM d, yyyy')}
        </span>
      )
    },
    enableSorting: true,
    size: 140,
  },
]
