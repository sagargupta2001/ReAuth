import type { ColumnDef } from '@tanstack/react-table'
import { Copy, MoreHorizontal } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import type { OidcClient } from '@/entities/oidc/model/types'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

export const clientColumns: ColumnDef<OidcClient>[] = [
  {
    accessorKey: 'client_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Client ID" />,
    cell: ({ row }) => <div className="font-mono font-medium">{row.getValue('client_id')}</div>,
    enableSorting: true, // Enable server-side sorting for this col
  },
  {
    accessorKey: 'redirect_uris',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Redirect URIs" />,
    cell: ({ row }) => {
      const raw = row.getValue('redirect_uris') as string
      try {
        // Parse the JSON string from the DB
        const uris = JSON.parse(raw) as string[]
        return (
          <div className="flex flex-col gap-1">
            {uris.map((uri) => (
              <span
                key={uri}
                className="text-muted-foreground max-w-[300px] truncate font-mono text-xs"
                title={uri}
              >
                {uri}
              </span>
            ))}
          </div>
        )
      } catch {
        return <span className="text-destructive text-xs">Invalid Data</span>
      }
    },
    enableSorting: false,
  },
  {
    accessorKey: 'scopes',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Scopes" />,
    cell: ({ row }) => {
      const scopes = (row.getValue('scopes') as string).split(' ')
      return (
        <div className="flex flex-wrap gap-1">
          {scopes.map((scope) => (
            <Badge key={scope} variant="outline" className="text-xs font-normal">
              {scope}
            </Badge>
          ))}
        </div>
      )
    },
    enableSorting: false,
  },
  {
    id: 'actions',
    cell: ({ row }) => {
      const client = row.original
      return (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-8 w-8 p-0">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuLabel>Actions</DropdownMenuLabel>
            <DropdownMenuItem
              onClick={() => {
                void navigator.clipboard.writeText(client.client_id)
                toast.success('Copied Client ID')
              }}
            >
              <Copy className="mr-2 h-4 w-4" /> Copy ID
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )
    },
  },
]
