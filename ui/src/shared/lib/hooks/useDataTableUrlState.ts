import { useCallback, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import { type PaginationState, type SortingState } from '@tanstack/react-table'
import { serializeFilterValue, type DataTableFilterValue } from '@/shared/ui/data-table/types'

interface DataTableUrlState {
  pagination: PaginationState
  sorting: SortingState
  searchTerm: string
  activeFilters: DataTableFilterValue[]
}

export function useDataTableUrlState(defaultSortBy = 'created_at', defaultSortDir: 'asc' | 'desc' = 'desc') {
  const [searchParams, setSearchParams] = useSearchParams()

  const state = useMemo((): DataTableUrlState => {
    const page = Number(searchParams.get('page')) || 1
    const perPage = Number(searchParams.get('per_page')) || 10
    const sortBy = searchParams.get('sort_by') || defaultSortBy
    const sortDir = (searchParams.get('sort_dir') as 'asc' | 'desc') || defaultSortDir
    const searchTerm = searchParams.get('q') || ''

    const activeFilters: DataTableFilterValue[] = []
    searchParams.forEach((value, key) => {
      if (key.startsWith('filter_')) {
        const filterKey = key.replace('filter_', '')
        const shouldParseJson = value.startsWith('{') || value.startsWith('[')
        if (shouldParseJson) {
          try {
            activeFilters.push({
              key: filterKey,
              value: JSON.parse(value),
            })
            return
          } catch {
            // Keep raw value if the payload is malformed JSON.
          }
        }
        activeFilters.push({
          key: filterKey,
          value,
        })
      }
    })

    return {
      pagination: {
        pageIndex: page - 1,
        pageSize: perPage,
      },
      sorting: [{ id: sortBy, desc: sortDir === 'desc' }],
      searchTerm,
      activeFilters,
    }
  }, [searchParams, defaultSortBy, defaultSortDir])

  const setPagination = useCallback(
    (updater: PaginationState | ((prev: PaginationState) => PaginationState)) => {
      const nextState = typeof updater === 'function' ? updater(state.pagination) : updater
      const params = new URLSearchParams(searchParams)
      params.set('page', String(nextState.pageIndex + 1))
      params.set('per_page', String(nextState.pageSize))
      setSearchParams(params)
    },
    [searchParams, setSearchParams, state.pagination],
  )

  const setSorting = useCallback(
    (updater: SortingState | ((prev: SortingState) => SortingState)) => {
      const nextState = typeof updater === 'function' ? updater(state.sorting) : updater
      const params = new URLSearchParams(searchParams)
      if (nextState.length > 0) {
        params.set('sort_by', nextState[0].id)
        params.set('sort_dir', nextState[0].desc ? 'desc' : 'asc')
      } else {
        params.delete('sort_by')
        params.delete('sort_dir')
      }
      setSearchParams(params)
    },
    [searchParams, setSearchParams, state.sorting],
  )

  const setSearchTerm = useCallback(
    (value: string) => {
      const params = new URLSearchParams(searchParams)
      if (value) {
        params.set('q', value)
        params.set('page', '1') // Reset page on search
      } else {
        params.delete('q')
      }
      setSearchParams(params)
    },
    [searchParams, setSearchParams],
  )

  const setActiveFilters = useCallback(
    (filters: DataTableFilterValue[]) => {
      const params = new URLSearchParams(searchParams)

      // Clear existing filters
      searchParams.forEach((_, key) => {
        if (key.startsWith('filter_')) {
          params.delete(key)
        }
      })

      // Add new filters
      filters.forEach((f) => {
        const value = serializeFilterValue(f.value)
        params.set(`filter_${f.key}`, value)
      })

      params.set('page', '1') // Reset page on filter change
      setSearchParams(params)
    },
    [searchParams, setSearchParams],
  )

  return {
    ...state,
    setPagination,
    setSorting,
    setSearchTerm,
    setActiveFilters,
  }
}
