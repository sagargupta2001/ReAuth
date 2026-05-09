import { FilterIcon, SearchIcon } from 'lucide-react'

import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Input } from '@/components/input'

import { DataTableFilterPills } from './filter-pills'
import {type DataTableToolbarProps } from './types'

export function DataTableToolbar<TData>({
  table,
  searchPlaceholder = 'Search...',
  searchValue,
  onSearch,
  customToolbarButtons,
  toolbarFilters,
  filters = [],
  activeFilters = [],
  onFilterChange,
}: DataTableToolbarProps<TData>) {
  const availableFilters = filters.filter((f) => !activeFilters.find((af) => af.key === f.key))

  const addFilter = (key: string) => {
    const field = filters.find((f) => f.key === key)
    if (!field || !onFilterChange) return
    onFilterChange([...activeFilters, { key, value: field.type === 'date-range' ? {} : '' }])
  }

  const handleClearAll = () => {
    table.resetColumnFilters()
    table.setGlobalFilter('')
    if (onSearch) onSearch('')
    if (onFilterChange) onFilterChange([])
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div className="flex flex-1 items-center space-x-2">
          <div className="relative flex items-center gap-2">
            <div className="relative">
              <SearchIcon
                aria-hidden="true"
                className="text-muted-foreground absolute top-1/2 left-2 -translate-y-1/2"
                size={16}
              />
              <Input
                placeholder={searchPlaceholder}
                value={searchValue ?? ''}
                onChange={(event) => onSearch?.(event.target.value)}
                className="h-8 w-[150px] pl-9 lg:w-[250px]"
              />
            </div>

            {filters.length > 0 && availableFilters.length > 0 && (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm" className="h-9 lg:px-3">
                    <FilterIcon className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-[180px]">
                  {availableFilters.map((filter) => (
                    <DropdownMenuItem key={filter.key} onClick={() => addFilter(filter.key)}>
                      {filter.label}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            )}

            {toolbarFilters}
          </div>
        </div>
        {customToolbarButtons}
      </div>

      {onFilterChange && (
        <DataTableFilterPills
          filters={filters}
          activeFilters={activeFilters}
          onFilterChange={onFilterChange}
          onClearAll={handleClearAll}
        />
      )}
    </div>
  )
}
