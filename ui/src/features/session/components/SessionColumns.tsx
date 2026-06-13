import { type ColumnDef } from '@tanstack/react-table'
import { formatDistanceToNow } from 'date-fns'
import { Laptop, Smartphone } from 'lucide-react'

import {
  deriveSessionStatus,
  isMobileUserAgent,
  parseUserAgent,
  sessionTypeLabel,
  statusBadge,
} from '@/entities/session/lib/session.logic'
import type { Session } from '@/entities/session/model/types'
import { SessionRowActions } from '@/features/session/components/SessionRowActions.tsx'
import { Badge } from '@/shared/ui/badge.tsx'
import { Checkbox } from '@/shared/ui/checkbox.tsx'
import { DataTableColumnHeader } from '@/shared/ui/data-table/column-header.tsx'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip.tsx'

export const getSessionColumns = (
  currentSessionId: string | undefined,
  onViewDetails: (session: Session) => void,
): ColumnDef<Session>[] => [
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
    size: 48,
  },
  {
    accessorKey: 'user_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="User" />,
    cell: ({ row }) => {
      const { username, email, user_id } = row.original
      const primary = username || user_id
      const secondary = username ? email || user_id : email
      return (
        <div className="flex min-w-0 flex-col">
          <span className="text-foreground truncate text-sm font-medium">{primary}</span>
          {secondary && (
            <span className="text-muted-foreground truncate font-mono text-[11px]">
              {secondary}
            </span>
          )}
        </div>
      )
    },
    size: 260,
  },
  {
    id: 'type',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Type" />,
    cell: ({ row }) => {
      const session = row.original
      const isOauth = !!session.client_id
      return (
        <Badge variant={isOauth ? 'outline' : 'muted'} className="max-w-[140px] truncate font-normal">
          {isOauth ? session.client_id : sessionTypeLabel(session)}
        </Badge>
      )
    },
    size: 150,
  },
  {
    id: 'device',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Device / Location" />,
    cell: ({ row }) => {
      const ua = row.original.user_agent
      const device = parseUserAgent(ua)
      const Icon = isMobileUserAgent(ua) ? Smartphone : Laptop
      return (
        <div className="flex items-center gap-2">
          <Icon className="text-muted-foreground h-4 w-4 shrink-0" />
          <div className="flex min-w-0 flex-col">
            <span className="truncate text-sm">{device.label}</span>
            <span className="text-muted-foreground font-mono text-[11px]">
              {row.original.ip_address || 'Unknown IP'}
            </span>
          </div>
        </div>
      )
    },
    size: 240,
  },
  {
    id: 'status',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Status" />,
    cell: ({ row }) => {
      const status = deriveSessionStatus(row.original, currentSessionId)
      const badge = statusBadge(status)
      return (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant={badge.variant} className="cursor-help">
                {badge.label}
              </Badge>
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-xs text-xs">{badge.hint}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )
    },
    size: 130,
  },
  {
    accessorKey: 'created_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Started" />,
    cell: ({ row }) => {
      const started = new Date(row.getValue('created_at'))
      const label = Number.isNaN(started.getTime())
        ? '—'
        : formatDistanceToNow(started, { addSuffix: true })
      return <span className="text-muted-foreground text-xs">{label}</span>
    },
    size: 150,
  },
  {
    id: 'actions',
    cell: ({ row }) => (
      <SessionRowActions
        session={row.original}
        currentSessionId={currentSessionId}
        onViewDetails={onViewDetails}
      />
    ),
    size: 50,
  },
]
