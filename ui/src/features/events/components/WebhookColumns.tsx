import type { ColumnDef } from '@tanstack/react-table'

import { Badge } from '@/components/badge'
import { formatRelativeTime } from '@/lib/utils'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

export type WebhookRow = {
  id: string
  name: string
  url: string
  description: string | null
  http_method: string
  status: 'active' | 'failing'
  subscriptions: string
  consecutive_failures: number
  last_fired_at: string | null
  updated_at: string
}

export const webhookColumns: ColumnDef<WebhookRow>[] = [
  {
    accessorKey: 'url',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Endpoint" />,
    cell: ({ row }) => (
      <div className="flex min-w-0 flex-col p-2">
        <span className="text-foreground truncate text-sm font-medium">{row.original.name}</span>
        <span className="text-muted-foreground truncate font-mono text-[11px]">
          {row.original.url}
        </span>
        {row.original.description ? (
          <span className="text-muted-foreground truncate text-[11px]">
            {row.original.description}
          </span>
        ) : null}
      </div>
    ),
    enableSorting: true,
    size: 340,
  },
  {
    accessorKey: 'http_method',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Method" />,
    cell: ({ row }) => (
      <Badge variant="outline" className="font-mono text-[11px] font-medium">
        {row.getValue('http_method')}
      </Badge>
    ),
    enableSorting: true,
    size: 110,
  },
  {
    accessorKey: 'status',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
    cell: ({ row }) => {
      const status = row.getValue('status') as WebhookRow['status']
      return (
        <div className="flex min-w-0 flex-col gap-1">
          <Badge variant={status === 'active' ? 'success' : 'destructive'} className="w-fit">
            {status === 'active' ? 'Active' : 'Failing'}
          </Badge>
          {row.original.consecutive_failures > 0 ? (
            <span className="text-muted-foreground text-xs">
              {row.original.consecutive_failures} consecutive failures
            </span>
          ) : null}
        </div>
      )
    },
    enableSorting: false,
    size: 160,
  },
  {
    accessorKey: 'subscriptions',
    header: 'Events',
    cell: ({ row }) => (
      <div className="text-muted-foreground line-clamp-2 text-sm">
        {row.getValue('subscriptions')}
      </div>
    ),
    enableSorting: false,
    size: 260,
  },
  {
    accessorKey: 'last_fired_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Last Fired" />,
    cell: ({ row }) => {
      const val = row.getValue('last_fired_at') as string | null
      return (
        <div className="text-muted-foreground text-sm">
          {val ? formatRelativeTime(val) : 'Never'}
        </div>
      )
    },
    enableSorting: true,
    size: 150,
  },
]
