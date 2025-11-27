import type { ComponentType } from 'react'

import { Cross2Icon } from '@radix-ui/react-icons'
import { type Table } from '@tanstack/react-table'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { DataTableFacetedFilter } from '@/shared/ui/data-table/faceted-filter.tsx'
import { DataTableViewOptions } from '@/shared/ui/data-table/view-options.tsx'

type DataTableToolbarProps<TData> = {
  table: Table<TData>
  searchPlaceholder?: string
  searchKey?: string
  searchValue?: string
  onSearch?: (value: string) => void
  filters?: {
    columnId: string
    title: string
    options: {
      label: string
      value: string
      icon?: ComponentType<{ className?: string }>
    }[]
  }[]
}

export function DataTableToolbar<TData>({
  table,
  searchPlaceholder = 'Filter...',
  searchKey,
  filters = [],
  searchValue,
  onSearch,
}: DataTableToolbarProps<TData>) {
  const isFiltered =
    table.getState().columnFilters.length > 0 ||
    table.getState().globalFilter ||
    (searchValue && searchValue.length > 0)

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2">
        {onSearch ? (
          // Case A: Server-Side Search (Controlled via Prop)
          <Input
            placeholder={searchPlaceholder}
            value={searchValue ?? ''}
            onChange={(event) => onSearch(event.target.value)}
            className="h-8 w-[150px] lg:w-[250px]"
          />
        ) : searchKey ? (
          // Case B: Client-Side Column Filter
          <Input
            placeholder={searchPlaceholder}
            value={(table.getColumn(searchKey)?.getFilterValue() as string) ?? ''}
            onChange={(event) => table.getColumn(searchKey)?.setFilterValue(event.target.value)}
            className="h-8 w-[150px] lg:w-[250px]"
          />
        ) : (
          // Case C: Client-Side Global Filter
          <Input
            placeholder={searchPlaceholder}
            value={table.getState().globalFilter ?? ''}
            onChange={(event) => table.setGlobalFilter(event.target.value)}
            className="h-8 w-[150px] lg:w-[250px]"
          />
        )}

        <div className="flex gap-x-2">
          {filters.map((filter) => {
            const column = table.getColumn(filter.columnId)
            if (!column) return null
            return (
              <DataTableFacetedFilter
                key={filter.columnId}
                column={column}
                title={filter.title}
                options={filter.options}
              />
            )
          })}
        </div>
        {isFiltered && (
          <Button
            variant="ghost"
            onClick={() => {
              table.resetColumnFilters()
              table.setGlobalFilter('')
              if (onSearch) onSearch('') // Clear server search too
            }}
            className="h-8 px-2 lg:px-3"
          >
            Reset
            <Cross2Icon className="ms-2 h-4 w-4" />
          </Button>
        )}
      </div>
      <DataTableViewOptions table={table} />
    </div>
  )
}
