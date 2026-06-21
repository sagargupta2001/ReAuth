import type { ColumnDef } from '@tanstack/react-table'
import { Globe, KeyRound } from 'lucide-react'

import type { OidcClient } from '@/entities/oidc/model/types.ts'
import { parseJsonArray } from '@/features/client/lib/clientFields'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

export const clientColumns: ColumnDef<OidcClient>[] = [
  {
    accessorKey: 'client_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Client ID" />,
    cell: ({ row }) => <div className="font-mono font-medium p-2">{row.getValue('client_id')}</div>,
    enableSorting: true, // Enable server-side sorting for this col
  },
  {
    accessorKey: 'redirect_uris',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Redirect URIs" />,
    cell: ({ row }) => {
      const count = parseJsonArray(row.getValue('redirect_uris') as string).length
      return (
        <div className="text-muted-foreground flex items-center gap-1.5 text-sm">
          <Globe className="h-3.5 w-3.5" />
          <span>
            {count} {count === 1 ? 'URI' : 'URIs'}
          </span>
        </div>
      )
    },
    enableSorting: false,
  },
  {
    accessorKey: 'scopes',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Scopes" />,
    cell: ({ row }) => {
      const count = parseJsonArray(row.getValue('scopes') as string).length
      return (
        <div className="text-muted-foreground flex items-center gap-1.5 text-sm">
          <KeyRound className="h-3.5 w-3.5" />
          <span>
            {count} {count === 1 ? 'Scope' : 'Scopes'}
          </span>
        </div>
      )
    },
    enableSorting: false,
  },
]
