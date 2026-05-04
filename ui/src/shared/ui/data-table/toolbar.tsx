import { type ComponentType, type ReactNode } from 'react'

import { Cross2Icon } from '@radix-ui/react-icons'
import { type Table } from '@tanstack/react-table'
import { SearchIcon } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { DataTableFacetedFilter } from '@/shared/ui/data-table/faceted-filter.tsx'

type DataTableToolbarProps<TData> = {
  table: Table<TData>
  searchPlaceholder?: string
  searchKey?: string
  searchValue?: string
  onSearch?: (value: string) => void
  customToolbarButtons?: ReactNode
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
  searchPlaceholder = 'Search...',
  searchKey,
  filters = [],
  searchValue,
  onSearch,
  customToolbarButtons,
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
          <div className="relative">
            <SearchIcon
              aria-hidden="true"
              className="absolute top-1/2 left-2 -translate-y-1/2 text-gray-500"
              size={16}
            />
            <Input
              placeholder={searchPlaceholder}
              value={searchValue ?? ''}
              onChange={(event) => onSearch(event.target.value)}
              className="h-8 w-[150px] lg:w-[250px] pl-9"
            />
          </div>
        ) : searchKey ? (
          // Case B: Client-Side Column Filter
          <div className="relative">
            <SearchIcon
              aria-hidden="true"
              className="absolute top-1/2 left-2 -translate-y-1/2 text-gray-500"
              size={16}
            />
            <Input
              placeholder={searchPlaceholder}
              value={(table.getColumn(searchKey)?.getFilterValue() as string) ?? ''}
              onChange={(event) => table.getColumn(searchKey)?.setFilterValue(event.target.value)}
              className="h-8 w-[150px] lg:w-[250px] pl-9"
            />
          </div>
        ) : (
          // Case C: Client-Side Global Filter
          <div className="relative">
            <SearchIcon
              aria-hidden="true"
              className="absolute top-1/2 left-2 -translate-y-1/2 text-gray-500"
              size={16}
            />

            <Input
              placeholder={searchPlaceholder}
              value={table.getState().globalFilter ?? ''}
              onChange={(event) => table.setGlobalFilter(event.target.value)}
              className="h-8 w-[150px] lg:w-[250px] pl-9"
            />
          </div>
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
              if (onSearch) onSearch('')
            }}
            className="h-8 px-2 lg:px-3"
          >
            Reset
            <Cross2Icon className="ms-2 h-4 w-4" />
          </Button>
        )}
      </div>
      {customToolbarButtons}
    </div>
  )
}
