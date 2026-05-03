import { type ColumnDef } from '@tanstack/react-table';
import { format } from 'date-fns';

import type { User } from '@/entities/user/model/types.ts';
import { Checkbox } from '@/shared/ui/checkbox.tsx';
import { DataTableColumnHeader } from '@/shared/ui/data-table';

export const userColumns: ColumnDef<User>[] = [
  {
    id: 'select',
    header: ({ table }) => (
      <div
        className="p-2"
        onClick={(e) => e.stopPropagation()}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <Checkbox
          checked={
            table.getIsAllPageRowsSelected() ||
            (table.getIsSomePageRowsSelected() && 'indeterminate')
          }
          onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
          aria-label="Select all"
          className="translate-y-0.5"
        />
      </div>
    ),
    cell: ({ row }) => (
      <div
        className="p-2"
        onClick={(e) => e.stopPropagation()}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <Checkbox
          checked={row.getIsSelected()}
          onCheckedChange={(value) => row.toggleSelected(!!value)}
          aria-label="Select row"
          className="translate-y-0.5"
        />
      </div>
    ),
    enableSorting: false,
    enableHiding: false,
    size: 20,
  },
  {
    id: 'user',
    accessorKey: 'username',
    header: ({ column }) => <DataTableColumnHeader column={column} title="User" />,
    cell: ({ row }) => {
      const username = row.getValue('user') as string
      const email = row.original.email
      return (
        <div className="flex flex-col">
          <span className="font-medium text-foreground">{username}</span>
          {email && <span className="text-xs text-muted-foreground">{email}</span>}
        </div>
      )
    },
    enableSorting: true,
  },
  {
    accessorKey: 'created_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Joined" />,
    cell: ({ row }) => {
      const value = row.getValue('created_at') as string | undefined
      if (!value) return <span className="text-muted-foreground text-sm">—</span>
      return <span className="text-muted-foreground text-sm">{format(new Date(value), 'MMM d, yyyy')}</span>
    },
    enableSorting: true,
  },
  {
    accessorKey: 'last_sign_in_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Last signed in" />,
    cell: ({ row }) => {
      const value = row.getValue('last_sign_in_at') as string | undefined
      if (!value) return <span className="text-muted-foreground text-sm">Never</span>
      return <span className="text-muted-foreground text-sm">{format(new Date(value), 'MMM d, yyyy')}</span>
    },
    enableSorting: true,
  },
]
