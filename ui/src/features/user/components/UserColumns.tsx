import { type ColumnDef } from '@tanstack/react-table';



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
    accessorKey: 'username',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Username" />,
    cell: ({ row }) => {
      return (
        <span className="text-muted-foreground font-mono text-xs">{row.getValue('username')}</span>
      )
    },
    enableSorting: true,
    meta: {
      align: 'center',
    },
  },
  {
    accessorKey: 'id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="User ID" />,
    cell: ({ row }) => (
      <div className="text-muted-foreground font-mono text-xs">{row.getValue('id')}</div>
    ),
    enableSorting: false,
  },
  {
    accessorKey: 'email',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Email" />,
    cell: ({ row }) => {
      const value = row.getValue('email') as string | null | undefined
      return <div className="text-muted-foreground text-sm">{value || '—'}</div>
    },
    enableSorting: true,
  },
]
