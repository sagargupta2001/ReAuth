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

function toDateOnly(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function normalizeDateLike(value: unknown): string | undefined {
  if (value == null) return undefined
  const date = value instanceof Date ? value : new Date(String(value))
  if (Number.isNaN(date.getTime())) return undefined
  return toDateOnly(date)
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
