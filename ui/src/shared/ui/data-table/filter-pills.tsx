import { X } from 'lucide-react'
import { type DateRange } from 'react-day-picker'

import { Button } from '@/shared/ui/button'
import { DateRangePicker } from '@/shared/ui/date-range-picker'
import { Input } from '@/shared/ui/input'

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

  const removeFilter = (key: string) => {
    onFilterChange(activeFilters.filter((f) => f.key !== key))
  }

  const updateFilterValue = (key: string, value: unknown) => {
    onFilterChange(activeFilters.map((f) => (f.key === key ? { ...f, value } : f)))
  }

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
  if (field.type === 'text') {
    return (
      <Input
        className="h-7 w-full border-0 bg-transparent p-0 px-1 text-sm focus-visible:ring-0 focus-visible:ring-offset-0"
        placeholder={field.placeholder || 'Enter value...'}
        value={(value as string) || ''}
        onChange={(e) => onChange(e.target.value)}
      />
    )
  }

  if (field.type === 'date-range') {
    const dateValue = value as DateRange | undefined

    return (
      <DateRangePicker
        initialDateFrom={dateValue?.from}
        initialDateTo={dateValue?.to}
        onUpdate={(values) => onChange(values.range)}
        showCompare={false}
        align="start"
        triggerClassName="h-7 border-0 bg-transparent px-1 text-xs shadow-none hover:bg-transparent"
      />
    )
  }

  return null
}
