import { useState } from 'react'

import {
  type ColumnDef,
  type ColumnFiltersState,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table'

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/table'
import { DataTableBulkActions } from '@/shared/ui/data-table/bulk-actions.tsx'
import { DataTablePagination } from '@/shared/ui/data-table/pagination.tsx'
import { DataTableToolbar } from '@/shared/ui/data-table/toolbar.tsx'

interface DataTableProps<TData, TValue> {
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
}

export function DataTable<TData, TValue>({
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
    // -----------------------

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

  return (
    <div className="space-y-4">
      <DataTableToolbar
        table={table}
        searchKey={searchKey}
        searchPlaceholder={searchPlaceholder}
        searchValue={searchValue}
        onSearch={onSearch}
      />
      <div className="rounded-md border">
        <Table>
          <TableHeader>
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
              table.getRowModel().rows.map((row) => (
                <TableRow key={row.id} data-state={row.getIsSelected() && 'selected'}>
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
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <DataTableBulkActions table={table} />
      <DataTablePagination table={table} />
    </div>
  )
}
