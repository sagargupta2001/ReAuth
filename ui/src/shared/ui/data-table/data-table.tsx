import { Fragment, type ReactNode, useState } from 'react'

import {
  type ColumnDef,
  type ColumnFiltersState,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
  type Table as Tb,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table'

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/table'
import { cn } from '@/lib/utils'
import { DataTableBulkActions } from '@/shared/ui/data-table/bulk-actions.tsx'
import { DataTablePagination } from '@/shared/ui/data-table/pagination.tsx'
import { DataTableToolbar } from '@/shared/ui/data-table/toolbar.tsx'

interface DataTableProps<TData, TValue> {
  onRowClick?: (row: TData) => void

  columns: ColumnDef<TData, TValue>[]
  data: TData[]

  // --- NEW PROPS FOR SERVER-SIDE CONTROL ---
  pageCount: number
  pagination?: PaginationState
  onPaginationChange?: OnChangeFn<PaginationState>
  sorting?: SortingState
  onSortingChange?: OnChangeFn<SortingState>
  // ---------------------------------------

  searchKey?: string
  searchPlaceholder?: string

  searchValue?: string
  onSearch?: (value: string) => void

  bulkEntityName?: string
  renderBulkActions?: (table: Tb<TData>) => ReactNode

  showToolbar?: boolean
  rootClassName?: string
  className?: string
  pageSizeOptions?: number[]

  getRowClassName?: (row: TData) => string
  isRowExpanded?: (row: TData) => boolean
  renderSubRow?: (row: TData) => ReactNode
}

export function DataTable<TData, TValue>({
  onRowClick,
  columns,
  data,
  pageCount,
  pagination,
  onPaginationChange,
  sorting,
  onSortingChange,
  searchValue,
  onSearch,
  searchKey = 'name',
  searchPlaceholder = 'Filter...',
  bulkEntityName,
  renderBulkActions,
  showToolbar = true,
  rootClassName,
  className,
  pageSizeOptions,
  getRowClassName,
  isRowExpanded,
  renderSubRow,
}: DataTableProps<TData, TValue>) {
  // These remain client-side as they don't usually affect the API fetch
  const [rowSelection, setRowSelection] = useState({})
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({})
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])

  const table = useReactTable({
    data,
    columns,
    pageCount, // Pass the total page count from server
    state: {
      // If controlled props are passed, use them. Otherwise fallback (though we expect them).
      pagination: pagination ?? { pageIndex: 0, pageSize: 10 },
      sorting: sorting ?? [],
      columnVisibility,
      rowSelection,
      columnFilters,
    },

    // --- SERVER-SIDE FLAGS ---
    manualPagination: true,
    manualSorting: true,
    manualFiltering: true, // If you handle search on server

    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onColumnVisibilityChange: setColumnVisibility,
    onColumnFiltersChange: setColumnFilters,

    // Pass the parent's handlers
    onPaginationChange,
    onSortingChange,

    getCoreRowModel: getCoreRowModel(),
    // We remove getPaginationRowModel and getSortedRowModel
    // because the server returns already sorted/paginated data.
  })

  const bulkActions = renderBulkActions?.(table)

  return (
    <div className={cn('flex flex-col gap-4', rootClassName)}>
      {showToolbar ? (
        <DataTableToolbar
          table={table}
          searchKey={searchKey}
          searchPlaceholder={searchPlaceholder}
          searchValue={searchValue}
          onSearch={onSearch}
        />
      ) : null}
      <div className={cn('relative overflow-auto rounded-md border', className)}>
        <Table className="w-full table-fixed" noWrapper>
          <TableHeader className="bg-background sticky top-0 z-10 shadow-sm">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id} colSpan={header.colSpan}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(header.column.columnDef.header, header.getContext())}
                    </TableHead>
                  )
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => {
                const expanded = isRowExpanded?.(row.original) ?? false
                const rowClassName = getRowClassName?.(row.original)
                return (
                  <Fragment key={row.id}>
                    <TableRow
                      data-state={row.getIsSelected() && 'selected'}
                      onClick={() => onRowClick?.(row.original)}
                      className={cn(onRowClick && 'hover:cursor-pointer', rowClassName)}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </TableCell>
                      ))}
                    </TableRow>
                    {expanded && renderSubRow ? (
                      <TableRow>
                        <TableCell
                          colSpan={table.getVisibleLeafColumns().length}
                          className="p-0"
                        >
                          {renderSubRow(row.original)}
                        </TableCell>
                      </TableRow>
                    ) : null}
                  </Fragment>
                )
              })
            ) : (
              <TableRow>
                <TableCell
                  colSpan={table.getVisibleLeafColumns().length}
                  className="h-24 text-center"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      {bulkActions ? (
        <DataTableBulkActions table={table} entityName={bulkEntityName ?? 'item'}>
          {bulkActions}
        </DataTableBulkActions>
      ) : null}
      <DataTablePagination table={table} pageSizeOptions={pageSizeOptions} />
    </div>
  )
}
