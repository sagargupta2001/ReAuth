import type { ColumnDef } from '@tanstack/react-table'
import { Box } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Switch } from '@/components/switch'
import { cn } from '@/lib/utils'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

import type { PluginStatus } from '@/entities/plugin/model/types'

export type PluginRow = {
  id: string
  name: string
  version: string
  status: PluginStatus
  events: string[]
}

type PluginColumnsOptions = {
  onToggle: (id: string, enabled: boolean) => void
  isPending: boolean
}

export function createPluginColumns({
  onToggle,
  isPending,
}: PluginColumnsOptions): ColumnDef<PluginRow>[] {
  return [
    {
      accessorKey: 'name',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Plugin Name" />,
      cell: ({ row }) => (
        <div className="flex items-center gap-2 font-medium">
          <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-primary">
            <Box className="h-4 w-4" />
          </span>
          {row.getValue('name')}
        </div>
      ),
      enableSorting: true,
    },
    {
      accessorKey: 'version',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Version" />,
      cell: ({ row }) => <span className="text-sm text-muted-foreground">{row.getValue('version')}</span>,
      enableSorting: true,
    },
    {
      accessorKey: 'status',
      header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
      cell: ({ row }) => {
        const status = row.getValue('status') as PluginStatus
        const isFailed = typeof status === 'object' && status !== null && 'failed' in status
        const isActive = status === 'active'
        const statusLabel = isFailed ? 'Failed' : isActive ? 'Enabled' : 'Disabled'
        return (
          <div
            className="flex items-center gap-2"
            onClick={(event) => event.stopPropagation()}
          >
            <Switch
              checked={isActive}
              onCheckedChange={(checked) => onToggle(row.original.id, checked)}
              disabled={isPending}
            />
            <span
              className={cn(
                'text-xs font-medium',
                isFailed ? 'text-rose-500' : isActive ? 'text-emerald-500' : 'text-muted-foreground',
              )}
            >
              {statusLabel}
            </span>
          </div>
        )
      },
      enableSorting: true,
    },
    {
      accessorKey: 'events',
      header: 'Requested Events',
      cell: ({ row }) => {
        const events = row.getValue('events') as string[]
        if (!events.length) {
          return (
            <Badge variant="outline" className="bg-muted/40">
              No events
            </Badge>
          )
        }
        return (
          <div className="flex flex-wrap gap-2">
            {events.map((event) => (
              <Badge key={event} variant="outline" className="bg-muted/40">
                {event}
              </Badge>
            ))}
          </div>
        )
      },
      enableSorting: false,
    },
  ]
}
