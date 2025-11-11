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
  },
  {
    accessorKey: 'target',
    header: 'Target',
    cell: (info) => <span className="font-mono text-xs">{info.getValue() as string}</span>,
  },
]

interface LogTableProps {
  logs: LogEntry[]
  // We'll add filters later
}

export function LogTable({ logs }: LogTableProps) {
  const table = useReactTable({
    data: logs,
    columns,
    getCoreRowModel: getCoreRowModel(),
    // We'll add filters and pagination later
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
              <TableRow key={row.id}>
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
