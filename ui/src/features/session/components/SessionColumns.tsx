import { type ColumnDef } from '@tanstack/react-table'
import { Globe, Laptop, Smartphone, Trash2 } from 'lucide-react'

import { Badge } from '@/shared/ui/badge.tsx'
import { Button } from '@/shared/ui/button.tsx'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip.tsx'
import type { Session } from '@/entities/session/model/types.ts'
import { DataTableColumnHeader } from '@/shared/ui/data-table/column-header.tsx'

// Helper to identify device type
const DeviceIcon = ({ ua }: { ua?: string }) => {
  if (!ua) return <Globe className="text-muted-foreground h-4 w-4" />
  const lower = ua.toLowerCase()
  if (lower.includes('mobile') || lower.includes('android') || lower.includes('iphone')) {
    return <Smartphone className="text-muted-foreground h-4 w-4" />
  }
  return <Laptop className="text-muted-foreground h-4 w-4" />
}

export const getSessionColumns = (
  currentSessionId: string | undefined,
  onRevoke: (id: string) => void,
): ColumnDef<Session>[] => [
  {
    accessorKey: 'user_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="User ID" />,
    cell: ({ row }) => (
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground font-mono text-xs">{row.getValue('user_id')}</span>
        {/* Highlight Current Session */}
        {row.original.id === currentSessionId && (
          <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
            Current
          </Badge>
        )}
      </div>
    ),
    size: 300,
  },
  {
    accessorKey: 'client_id',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Client" />,
    cell: ({ row }) => {
      const client = row.original.client_id
      return client ? (
        <Badge variant="outline" className="font-mono font-normal">
          {client}
        </Badge>
      ) : (
        <span className="text-muted-foreground text-xs italic">Admin Console</span>
      )
    },
    size: 150,
  },
  {
    accessorKey: 'ip_address',
    header: ({ column }) => <DataTableColumnHeader column={column} title="IP Address" />,
    cell: ({ row }) => (
      <div className="font-mono text-xs">{row.getValue('ip_address') || 'Unknown'}</div>
    ),
    size: 140,
  },
  {
    accessorKey: 'user_agent',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Device / UA" />,
    cell: ({ row }) => {
      const ua = row.original.user_agent || 'Unknown'
      return (
        <div className="flex items-center gap-2">
          <DeviceIcon ua={ua} />
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="text-muted-foreground max-w-[250px] cursor-help truncate text-xs">
                  {ua}
                </span>
              </TooltipTrigger>
              <TooltipContent>
                <p className="max-w-xs text-xs break-words">{ua}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      )
    },
    size: 300,
  },
  {
    accessorKey: 'created_at',
    header: ({ column }) => <DataTableColumnHeader column={column} title="Started" />,
    cell: ({ row }) => {
      return (
        <span className="text-muted-foreground text-xs">
          {new Date(row.getValue('created_at')).toLocaleString()}
        </span>
      )
    },
    size: 180,
  },
  {
    id: 'actions',
    cell: ({ row }) => (
      <div className="flex justify-end">
        <Button
          variant="ghost"
          size="icon"
          className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 h-8 w-8"
          onClick={(e) => {
            e.stopPropagation()
            onRevoke(row.original.id)
          }}
          disabled={row.original.id === currentSessionId} // Prevent revoking own session easily
          title={
            row.original.id === currentSessionId
              ? 'Cannot revoke current session here'
              : 'Revoke Session'
          }
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    ),
    size: 50,
  },
]
