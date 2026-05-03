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
  pageCount: number
  pagination?: PaginationState
  onPaginationChange?: OnChangeFn<PaginationState>
  sorting?: SortingState
  onSortingChange?: OnChangeFn<SortingState>
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
  customToolbarButtons?: ReactNode
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
  customToolbarButtons,
}: DataTableProps<TData, TValue>) {
  const [rowSelection, setRowSelection] = useState({})
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({})
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])

  const table = useReactTable({
    data,
    columns,
    pageCount,
    state: {
      pagination: pagination ?? { pageIndex: 0, pageSize: 10 },
      sorting: sorting ?? [],
      columnVisibility,
      rowSelection,
      columnFilters,
    },

    manualPagination: true,
    manualSorting: true,
    manualFiltering: true,
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onColumnVisibilityChange: setColumnVisibility,
    onColumnFiltersChange: setColumnFilters,
    onPaginationChange,
    onSortingChange,
    getCoreRowModel: getCoreRowModel(),
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
          customToolbarButtons={customToolbarButtons}
        />
      ) : null}
      <div className={cn('relative overflow-auto rounded-2xl p-2 bg-[#171717]', className)}>
        <Table className="w-full table-fixed " noWrapper>
          <TableHeader className="sticky top-0 z-10  bg-[#171717]">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead
                      key={header.id}
                      colSpan={header.colSpan}
                      style={{ width: header.getSize() }}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(header.column.columnDef.header, header.getContext())}
                    </TableHead>
                  )
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody >
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => {
                const expanded = isRowExpanded?.(row.original) ?? false
                const rowClassName = getRowClassName?.(row.original)
                return (
                  <Fragment key={row.id}>
                    <TableRow
                      data-state={row.getIsSelected() && 'selected'}
                      onClick={() => onRowClick?.(row.original)}
                      className={cn(
                        onRowClick && 'bg-[#000000] hover:cursor-pointer',
                        rowClassName,
                      )}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <TableCell key={cell.id}>
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </TableCell>
                      ))}
                    </TableRow>
                    {expanded && renderSubRow ? (
                      <TableRow>
                        <TableCell colSpan={table.getVisibleLeafColumns().length}>
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
