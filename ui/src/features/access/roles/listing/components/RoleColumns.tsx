import type { ColumnDef } from '@tanstack/react-table'
import { MoreHorizontal, Shield } from 'lucide-react'

import { Button } from '@/shared/ui/button.tsx'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu.tsx'
import type { Role } from '@/entities/access/api/useRoles.ts'

export const roleColumns: ColumnDef<Role>[] = [
  {
    accessorKey: 'name',
    header: 'Name',
    cell: ({ row }) => {
      return (
        <div className="flex items-center gap-2 font-medium">
          <Shield className="text-muted-foreground h-4 w-4" />
          {row.getValue('name')}
        </div>
      )
    },
  },
  {
    accessorKey: 'description',
    header: 'Description',
    cell: ({ row }) => (
      <div className="text-muted-foreground max-w-[500px] truncate">
        {row.getValue('description') || '-'}
      </div>
    ),
  },
  {
    id: 'actions',
    cell: ({ row }) => {
      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-8 w-8 p-0">
              <span className="sr-only">Open menu</span>
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => console.log('Edit', row.original.id)}>
              Edit Role
            </DropdownMenuItem>
            <DropdownMenuItem
              className="text-destructive focus:text-destructive"
              onClick={() => console.log('Delete', row.original.id)}
            >
              Delete
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )
    },
  },
]
