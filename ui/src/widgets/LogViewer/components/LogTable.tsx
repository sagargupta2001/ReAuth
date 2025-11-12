import { useRef } from 'react'

import { type ColumnDef, flexRender, getCoreRowModel, useReactTable } from '@tanstack/react-table'
import { useVirtualizer } from '@tanstack/react-virtual'

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
  // Use the primary message if it exists
  if (log.message) return log.message

  // Fallback to a field named "message" or "msg"
  if (log.fields?.message) return log.fields.message

  if (log.fields?.msg) return log.fields.msg

  return 'No message, view fields for details.'
}

const columns: ColumnDef<LogEntry>[] = [
  {
    accessorKey: 'timestamp',
    header: 'Timestamp',
    cell: (info) => {
      const date = new Date(info.getValue() as string)
      // Optimized date formatting
      return date.toLocaleString('en-IN', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      })
    },
    size: 150,
  },
  {
    accessorKey: 'level',
    header: 'Level',
    cell: (info) => {
      const level = info.getValue() as string
      let variant: 'default' | 'destructive' | 'secondary' = 'secondary'
      if (level === 'ERROR') variant = 'destructive'
      if (level === 'INFO') variant = 'default'
      return <Badge variant={variant}>{level}</Badge>
    },
    size: 100,
  },
  {
    accessorKey: 'message',
    header: 'Message',
    cell: (info) => {
      const row = info.row.original
      return <span className="font-medium">{getDisplayMessage(row)}</span>
    },
    size: 350,
  },
  {
    accessorKey: 'target',
    header: 'Target',
    cell: (info) => <LogTarget target={info.getValue() as string} />,
    size: 450,
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

  const tableContainerRef = useRef<HTMLDivElement>(null) // Ref for the scrolling element
  const { rows } = table.getRowModel()
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 41, // Fixed row height
    overscan: 10,
  })

  const virtualRows = rowVirtualizer.getVirtualItems()
  const totalSize = rowVirtualizer.getTotalSize()
  const paddingTop = virtualRows.length > 0 ? (virtualRows[0]?.start ?? 0) : 0
  const paddingBottom =
    virtualRows.length > 0 ? totalSize - (virtualRows[virtualRows.length - 1]?.end ?? 0) : 0

  return (
    // This is the scrolling container with a fixed height
    <div
      ref={tableContainerRef}
      className="overflow-auto rounded-md border" // The container itself scrolls
      style={{ height: '600px' }}
    >
      <Table className="w-full table-fixed" noWrapper>
        <TableHeader className="bg-background sticky top-0 z-10">
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead
                  key={header.id}
                  style={{ width: `${header.getSize()}px` }}
                  className="whitespace-nowrap"
                >
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>

        <TableBody>
          {paddingTop > 0 && (
            <TableRow style={{ height: `${paddingTop}px` }}>
              <TableCell colSpan={columns.length} />
            </TableRow>
          )}

          {/* Visible rows */}
          {virtualRows.length === 0 ? (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No logs to display.
              </TableCell>
            </TableRow>
          ) : (
            virtualRows.map((virtualRow) => {
              const row = rows[virtualRow.index]
              return (
                <TableRow
                  key={row.id}
                  onClick={() => onRowClick(row.original)}
                  className="cursor-pointer"
                  data-index={virtualRow.index}
                  style={{ height: `${virtualRow.size}px` }} // Set row height
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} style={{ width: `${cell.column.getSize()}px` }}>
                      <div className="overflow-hidden text-ellipsis whitespace-nowrap">
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </div>
                    </TableCell>
                  ))}
                </TableRow>
              )
            })
          )}

          {/* Bottom padding row */}
          {paddingBottom > 0 && (
            <TableRow style={{ height: `${paddingBottom}px` }}>
              <TableCell colSpan={columns.length} />
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  )
}
