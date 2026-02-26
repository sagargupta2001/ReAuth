import type { ColumnDef } from '@tanstack/react-table'
import type { TFunction } from 'i18next'

import { Badge } from '@/components/badge'
import { DataTableColumnHeader } from '@/shared/ui/data-table'

import type { TelemetryLog } from '../model/types'

type LogColumnsOptions = {
  t: TFunction<'logs'>
  onSelectTrace: (traceId: string) => void
}

function getDisplayMessage(log: TelemetryLog): string {
  if (log.message) return log.message
  if (typeof log.fields.message === 'string') return log.fields.message
  if (typeof log.fields.msg === 'string') return log.fields.msg
  return 'No message, view metadata for details.'
}

function getRequestLabel(log: TelemetryLog): string {
  if (log.method && log.path) {
    return `${log.method} ${log.path}`
  }
  if (log.method && log.route) {
    return `${log.method} ${log.route}`
  }
  if (log.route) return log.route
  if (log.path) return log.path
  return getDisplayMessage(log)
}

function formatDuration(duration?: number | null) {
  if (duration === null || duration === undefined) return '—'
  if (!Number.isFinite(duration)) return '—'
  if (duration >= 1000) return `${(duration / 1000).toFixed(2)}s`
  return `${duration}ms`
}

function formatIdentifier(value?: string | null) {
  if (!value) return '—'
  if (value.length <= 12) return value
  return `${value.slice(0, 8)}…`
}

function statusBadgeVariant(status?: number | null) {
  if (status === null || status === undefined) return 'muted'
  if (status >= 500) return 'destructive'
  if (status >= 400) return 'warning'
  if (status >= 300) return 'secondary'
  return 'success'
}

function formatTimestamp(timestamp: string) {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  return date.toLocaleString('en-US', {
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

function levelBadgeVariant(level: string) {
  switch (level) {
    case 'ERROR':
      return 'destructive'
    case 'WARN':
      return 'secondary'
    case 'INFO':
      return 'default'
    default:
      return 'secondary'
  }
}

export function createLogColumns({ t, onSelectTrace }: LogColumnsOptions): ColumnDef<TelemetryLog>[] {
  return [
    {
      accessorKey: 'timestamp',
      header: ({ column }) => (
        <DataTableColumnHeader column={column} title={t('LOGS_TABLE.TIMESTAMP')} />
      ),
      cell: ({ row }) => (
        <div className="font-mono text-xs text-muted-foreground">
          {formatTimestamp(row.getValue('timestamp') as string)}
        </div>
      ),
      enableSorting: true,
    },
    {
      accessorKey: 'level',
      header: t('LOGS_TABLE.LEVEL'),
      cell: ({ row }) => (
        <Badge variant={levelBadgeVariant(row.getValue('level') as string)} className="text-xs">
          {row.getValue('level') as string}
        </Badge>
      ),
      enableSorting: false,
    },
    {
      id: 'request',
      header: t('LOGS_TABLE.REQUEST'),
      cell: ({ row }) => {
        const log = row.original
        const requestLabel = getRequestLabel(log)
        return (
          <div className="flex flex-col gap-1 text-sm">
            <span className="font-medium text-foreground">{requestLabel}</span>
            <span className="text-xs text-muted-foreground truncate">
              {log.route && log.route !== log.path ? log.route : log.target}
            </span>
          </div>
        )
      },
      enableSorting: false,
    },
    {
      accessorKey: 'status',
      header: t('LOGS_TABLE.STATUS'),
      cell: ({ row }) => {
        const status = row.getValue('status') as number | null | undefined
        if (!status) {
          return <span className="text-xs text-muted-foreground">—</span>
        }
        return (
          <Badge variant={statusBadgeVariant(status)} className="text-xs">
            {status}
          </Badge>
        )
      },
      enableSorting: false,
    },
    {
      accessorKey: 'duration_ms',
      header: ({ column }) => (
        <DataTableColumnHeader column={column} title={t('LOGS_TABLE.DURATION')} />
      ),
      cell: ({ row }) => (
        <div className="font-mono text-xs text-muted-foreground">
          {formatDuration(row.getValue('duration_ms') as number | null | undefined)}
        </div>
      ),
      enableSorting: true,
    },
    {
      accessorKey: 'trace_id',
      header: t('LOGS_TABLE.TRACE_ID'),
      cell: ({ row }) => {
        const traceId = row.getValue('trace_id') as string | null | undefined
        if (!traceId) {
          return <span className="text-xs text-muted-foreground">—</span>
        }
        return (
          <button
            className="font-mono text-xs text-sky-400 hover:text-sky-300"
            onClick={(event) => {
              event.stopPropagation()
              onSelectTrace(traceId)
            }}
          >
            {traceId}
          </button>
        )
      },
      enableSorting: false,
    },
    {
      accessorKey: 'user_id',
      header: t('LOGS_TABLE.USER'),
      cell: ({ row }) => (
        <div
          className="font-mono text-xs text-muted-foreground"
          title={row.getValue('user_id') ?? undefined}
        >
          {formatIdentifier(row.getValue('user_id') as string | null | undefined)}
        </div>
      ),
      enableSorting: false,
    },
    {
      accessorKey: 'realm',
      header: t('LOGS_TABLE.REALM'),
      cell: ({ row }) => (
        <div className="text-xs text-muted-foreground">
          {row.getValue('realm') ? String(row.getValue('realm')) : '—'}
        </div>
      ),
      enableSorting: false,
    },
  ]
}
