import { Check, ChevronsUpDown, X } from 'lucide-react'

import { Button } from '@/shared/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/shared/ui/command'
import { DateTimeRangePicker } from '@/shared/ui/date-time-range-picker'
import { Input } from '@/shared/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/shared/ui/popover'
import { cn } from '@/shared/lib/utils'

import { type DataTableFilterField, type DataTableFilterValue } from './types'

interface DataTableFilterPillsProps {
  filters: DataTableFilterField[]
  activeFilters: DataTableFilterValue[]
  onFilterChange: (filters: DataTableFilterValue[]) => void
  onClearAll: () => void
}

export function DataTableFilterPills({
  filters,
  activeFilters,
  onFilterChange,
  onClearAll,
}: DataTableFilterPillsProps) {
  if (activeFilters.length === 0) return null

  const removeFilter = (key: string) => onFilterChange(activeFilters.filter((f) => f.key !== key))

  const updateFilterValue = (key: string, value: unknown) =>
    onFilterChange(activeFilters.map((f) => (f.key === key ? { ...f, value } : f)))

  return (
    <div className="flex flex-wrap items-center gap-2 pb-2">
      {activeFilters.map((activeFilter) => {
        const field = filters.find((f) => f.key === activeFilter.key)
        if (!field) return null

        return (
          <div
            key={activeFilter.key}
            className="bg-background flex items-center divide-x overflow-hidden rounded-full border text-sm shadow-sm"
          >
            <div className="bg-muted/50 flex items-center gap-1 px-3 py-1">
              <span className="text-muted-foreground font-medium">{field.label}</span>
              <button
                onClick={() => removeFilter(activeFilter.key)}
                className="hover:bg-muted text-muted-foreground ml-1 rounded-full p-0.5"
              >
                <X size={12} />
              </button>
            </div>
            <div className="flex min-w-[120px] items-center px-2 py-0.5">
              <FilterValueControl
                field={field}
                value={activeFilter.value}
                onChange={(val) => updateFilterValue(activeFilter.key, val)}
              />
            </div>
          </div>
        )
      })}
      <Button
        variant="link"
        size="sm"
        onClick={onClearAll}
        className="text-muted-foreground hover:text-foreground h-7 px-2"
      >
        Clear filters
      </Button>
    </div>
  )
}

function FilterValueControl({
  field,
  value,
  onChange,
}: {
  field: DataTableFilterField
  value: unknown
  onChange: (value: unknown) => void
}) {
  if (field.type === 'text')
    return (
      <Input
        className="h-7 w-full border-0 shadow-none bg-transparent p-0 px-1 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
        placeholder={field.placeholder || 'Enter value...'}
        value={(value as string) || ''}
        onChange={(e) => onChange(e.target.value)}
      />
    )

  if (field.type === 'date-range') {
    const dateValue = value as { from?: Date | string; to?: Date | string } | undefined

    return (
      <DateTimeRangePicker
        value={dateValue}
        onChange={(range) => onChange(range)}
        align="start"
        placeholder="Pick range…"
        triggerClassName="h-7 border-0 bg-transparent px-1 text-xs shadow-none hover:bg-transparent"
      />
    )
  }

  if (field.type === 'select') {
    return <SelectFilterControl field={field} value={value} onChange={onChange} />
  }

  return null
}

function SelectFilterControl({
  field,
  value,
  onChange,
}: {
  field: DataTableFilterField
  value: unknown
  onChange: (value: unknown) => void
}) {
  const options = field.options ?? []
  const selected = options.find((option) => option.value === value)

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          role="combobox"
          className="h-7 justify-between gap-1 border-0 px-1 text-sm font-normal shadow-none hover:bg-transparent"
        >
          {selected?.label ?? <span className="text-muted-foreground">Select...</span>}
          <ChevronsUpDown className="size-3.5 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[220px] p-0" align="start">
        <Command>
          <CommandInput placeholder={field.placeholder ?? `Search ${field.label.toLowerCase()}...`} />
          <CommandList>
            <CommandEmpty>No results found.</CommandEmpty>
            <CommandGroup>
              {options.map((option) => (
                <CommandItem
                  key={option.value}
                  value={option.label}
                  onSelect={() => onChange(option.value === value ? '' : option.value)}
                >
                  <Check
                    className={cn(
                      'size-4',
                      option.value === value ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                  {option.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
