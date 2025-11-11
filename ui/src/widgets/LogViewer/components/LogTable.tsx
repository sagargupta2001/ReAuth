import { type ColumnDef, flexRender, getCoreRowModel, useReactTable } from '@tanstack/react-table'

import { Badge } from '@/components/badge'
import type { LogEntry } from '@/entities/log/model/types'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/shared/ui/table.tsx'
import { LogTarget } from '@/widgets/LogViewer/components/LogTarget.tsx'

// A helper function to find the "best" message
function getDisplayMessage(log: LogEntry): string {
  // 1. Use the primary message if it exists
  if (log.message) {
    return log.message
  }
  // 2. Fallback to a field named "message" or "msg"
  // This will now work because `log.fields` is an object.
  if (log.fields?.message) {
    return log.fields.message
  }
  if (log.fields?.msg) {
    return log.fields.msg
  }
  // 3. Fallback to a generic message
  return 'No message, view fields for details.'
}

// 1. Define the columns for your table
const columns: ColumnDef<LogEntry>[] = [
  {
    accessorKey: 'timestamp',
    header: 'Timestamp',
    cell: (info) => new Date(info.getValue() as string).toLocaleString(),
  },
  {
    accessorKey: 'level',
    header: 'Level',
    cell: (info) => {
      const level = info.getValue() as string
      let variant: 'default' | 'destructive' | 'secondary' = 'secondary'
      if (level === 'ERROR') variant = 'destructive'
      if (level === 'INFO') variant = 'default' // default is blue/primary
      return <Badge variant={variant}>{level}</Badge>
    },
  },
  {
    accessorKey: 'message',
    header: 'Message',
    cell: (info) => {
      const row = info.row.original
      return <span className="font-medium">{getDisplayMessage(row)}</span>
    },
  },
  {
    accessorKey: 'target',
    header: 'Target',
    cell: (info) => <LogTarget target={info.getValue() as string} />,
  },
]

interface LogTableProps {
  logs: LogEntry[]
  onRowClick: (log: LogEntry) => void
}

export function LogTable({ logs, onRowClick }: LogTableProps) {
  const table = useReactTable({
    data: logs,
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id}>
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                className="cursor-pointer"
                key={row.id}
                onClick={() => onRowClick(row.original)}
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id}>
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No logs to display.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  )
}
