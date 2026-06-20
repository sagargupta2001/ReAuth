import { type Table } from '@tanstack/react-table'

export type FilterType = 'text' | 'date-range' | 'select'

export interface DataTableFilterOption {
  value: string
  label: string
}

export interface DataTableFilterField {
  key: string
  label: string
  type: FilterType
  options?: DataTableFilterOption[] // For 'select' type
  placeholder?: string
}

export interface DataTableFilterValue {
  key: string
  value: unknown // string for text, { from: Date, to: Date } for date-range
}

interface DateRangeLike {
  from?: unknown
  to?: unknown
}

// Serialize to full ISO timestamps so the time component survives the round-trip.
// The backend filters accept both RFC3339 datetimes and plain YYYY-MM-DD dates.
function normalizeDateLike(value: unknown): string | undefined {
  if (value == null) return undefined
  const date = value instanceof Date ? value : new Date(String(value))
  if (Number.isNaN(date.getTime())) return undefined
  return date.toISOString()
}

export function serializeFilterValue(value: unknown): string {
  if (typeof value === 'string') return value

  if (typeof value === 'object' && value !== null) {
    const maybeDateRange = value as DateRangeLike
    if ('from' in maybeDateRange || 'to' in maybeDateRange) {
      return JSON.stringify({
        from: normalizeDateLike(maybeDateRange.from),
        to: normalizeDateLike(maybeDateRange.to),
      })
    }
  }

  return JSON.stringify(value) ?? ''
}

export interface DataTableToolbarProps<TData> {
  table: Table<TData>
  searchKey?: string
  searchPlaceholder?: string
  searchValue?: string
  onSearch?: (value: string) => void
  customToolbarButtons?: React.ReactNode
  toolbarFilters?: React.ReactNode
  filters?: DataTableFilterField[]
  activeFilters?: DataTableFilterValue[]
  onFilterChange?: (filters: DataTableFilterValue[]) => void
}
