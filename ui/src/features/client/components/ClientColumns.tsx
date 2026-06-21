import type { ColumnDef } from '@tanstack/react-table'

import type { OidcClient } from '@/entities/oidc/model/types.ts'
import { Badge } from '@/shared/ui/badge.tsx'
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
]
