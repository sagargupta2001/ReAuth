import type { ColumnDef } from '@tanstack/react-table'

import { Badge } from '@/components/badge'
import { formatRelativeTime } from '@/lib/utils'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

export type WebhookRow = {
  id: string
  url: string
  http_method: string
  status: 'active' | 'failing'
  subscriptions: string
  last_fired_at: string | null
  updated_at: string
}

export const webhookColumns: ColumnDef<WebhookRow>[] = [
  {
    accessorKey: 'http_method',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Method" />,
    cell: ({ row }) => (
      <div className="font-mono text-xs text-muted-foreground">
        {row.getValue('http_method')}
      </div>
    ),
    enableSorting: true,
  },
  {
    accessorKey: 'url',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Endpoint URL" />,
    cell: ({ row }) => (
      <div className="font-mono text-xs text-muted-foreground">{row.getValue('url')}</div>
    ),
    enableSorting: true,
  },
  {
    accessorKey: 'status',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
    cell: ({ row }) => {
      const status = row.getValue('status') as WebhookRow['status']
      return (
        <Badge variant={status === 'active' ? 'success' : 'destructive'}>
          {status === 'active' ? 'Active' : 'Failing'}
        </Badge>
      )
    },
    enableSorting: false,
  },
  {
    accessorKey: 'subscriptions',
    header: 'Subscriptions',
    cell: ({ row }) => (
      <div className="text-sm text-muted-foreground">{row.getValue('subscriptions')}</div>
    ),
    enableSorting: false,
  },
  {
    accessorKey: 'last_fired_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Last Fired" />,
    cell: ({ row }) => {
      const val = row.getValue('last_fired_at') as string | null
      return (
        <div className="text-sm text-muted-foreground">
          {val ? formatRelativeTime(val) : 'Never'}
        </div>
      )
    },
    enableSorting: true,
  },
]
