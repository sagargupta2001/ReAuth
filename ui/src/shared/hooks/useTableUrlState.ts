import { useMemo, useState } from 'react'

import type { ColumnFiltersState, OnChangeFn, PaginationState } from '@tanstack/react-table'

type SearchRecord = Record<string, unknown>

function buildQuery(
  prev: SearchRecord,
  patch: Record<string, unknown>,
): Record<string, string | undefined> {
  const output: Record<string, string | undefined> = {}

  for (const [k, v] of Object.entries({ ...prev, ...patch })) {
    if (v === undefined || v === null || v === '') continue
    output[k] = String(v)
  }

  return output
}

export type NavigateFn = (opts: {
  search: Record<string, string | undefined> | URLSearchParams | string
  replace?: boolean
}) => void

type UseTableUrlStateParams = {
  search: SearchRecord
  navigate: NavigateFn
  pagination?: {
    pageKey?: string
    pageSizeKey?: string
    defaultPage?: number
    defaultPageSize?: number
  }
  globalFilter?: {
    enabled?: boolean
    key?: string
    trim?: boolean
  }
  columnFilters?: Array<
    | {
        columnId: string
        searchKey: string
        type?: 'string'
        // Optional transformers for custom types
        serialize?: (value: unknown) => unknown
        deserialize?: (value: unknown) => unknown
      }
    | {
        columnId: string
        searchKey: string
        type: 'array'
        serialize?: (value: unknown) => unknown
        deserialize?: (value: unknown) => unknown
      }
  >
}

type UseTableUrlStateReturn = {
  // Global filter
  globalFilter?: string
  onGlobalFilterChange?: OnChangeFn<string>
  // Column filters
  columnFilters: ColumnFiltersState
  onColumnFiltersChange: OnChangeFn<ColumnFiltersState>
  // Pagination
  pagination: PaginationState
  onPaginationChange: OnChangeFn<PaginationState>
  // Helpers
  ensurePageInRange: (pageCount: number, opts?: { resetTo?: 'first' | 'last' }) => void
}

export function useTableUrlState(params: UseTableUrlStateParams): UseTableUrlStateReturn {
  const {
    search,
    navigate,
    pagination: paginationCfg,
    globalFilter: globalFilterCfg,
    columnFilters: columnFiltersCfg = [],
  } = params

  const pageKey = paginationCfg?.pageKey ?? ('page' as string)
  const pageSizeKey = paginationCfg?.pageSizeKey ?? ('pageSize' as string)
  const defaultPage = paginationCfg?.defaultPage ?? 1
  const defaultPageSize = paginationCfg?.defaultPageSize ?? 10

  const globalFilterKey = globalFilterCfg?.key ?? ('filter' as string)
  const globalFilterEnabled = globalFilterCfg?.enabled ?? true
  const trimGlobal = globalFilterCfg?.trim ?? true

  // Build initial column filters from the current search params
  const initialColumnFilters: ColumnFiltersState = useMemo(() => {
    const collected: ColumnFiltersState = []
    for (const cfg of columnFiltersCfg) {
      const raw = (search as SearchRecord)[cfg.searchKey]
      const deserialize = cfg.deserialize ?? ((v: unknown) => v)
      if (cfg.type === 'string') {
        const value = (deserialize(raw) as string) ?? ''
        if (value.trim() !== '') collected.push({ id: cfg.columnId, value })
      } else {
        // default to array type
        const value = (deserialize(raw) as unknown[]) ?? []
        if (Array.isArray(value) && value.length > 0) collected.push({ id: cfg.columnId, value })
      }
    }
    return collected
  }, [columnFiltersCfg, search])

  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>(initialColumnFilters)

  const pagination: PaginationState = useMemo(() => {
    const rawPage = (search as SearchRecord)[pageKey]
    const rawPageSize = (search as SearchRecord)[pageSizeKey]
    const pageNum = typeof rawPage === 'number' ? rawPage : defaultPage
    const pageSizeNum = typeof rawPageSize === 'number' ? rawPageSize : defaultPageSize
    return { pageIndex: Math.max(0, pageNum - 1), pageSize: pageSizeNum }
  }, [search, pageKey, pageSizeKey, defaultPage, defaultPageSize])

  const onPaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination) : updater

    const nextPage = next.pageIndex + 1
    const nextPageSize = next.pageSize

    const patch: Record<string, unknown> = {
      [pageKey]: nextPage <= defaultPage ? undefined : nextPage,
      [pageSizeKey]: nextPageSize === defaultPageSize ? undefined : nextPageSize,
    }

    navigate({
      search: buildQuery(search, patch),
    })
  }

  const [globalFilter, setGlobalFilter] = useState<string | undefined>(() => {
    if (!globalFilterEnabled) return undefined
    const raw = (search as SearchRecord)[globalFilterKey]
    return typeof raw === 'string' ? raw : ''
  })

  const onGlobalFilterChange: OnChangeFn<string> | undefined = globalFilterEnabled
    ? (updater) => {
        const next = typeof updater === 'function' ? updater(globalFilter ?? '') : updater
        const value = trimGlobal ? next.trim() : next

        setGlobalFilter(value)

        const patch: Record<string, unknown> = {
          [pageKey]: undefined,
          [globalFilterKey]: value || undefined,
        }

        navigate({
          search: buildQuery(search, patch),
        })
      }
    : undefined

  const onColumnFiltersChange: OnChangeFn<ColumnFiltersState> = (updater) => {
    const next = typeof updater === 'function' ? updater(columnFilters) : updater
    setColumnFilters(next)

    const patch: Record<string, unknown> = {}

    for (const cfg of columnFiltersCfg) {
      const found = next.find((f) => f.id === cfg.columnId)
      const serialize = cfg.serialize ?? ((v: unknown) => v)

      if (cfg.type === 'string') {
        const value = typeof found?.value === 'string' ? found.value : ''
        patch[cfg.searchKey] = value.trim() ? serialize(value) : undefined
      } else {
        const value = Array.isArray(found?.value) ? found.value : []
        patch[cfg.searchKey] = value.length ? serialize(value) : undefined
      }
    }

    patch[pageKey] = undefined // reset page

    navigate({
      search: buildQuery(search, patch),
    })
  }

  const ensurePageInRange = (
    pageCount: number,
    opts: { resetTo?: 'first' | 'last' } = { resetTo: 'first' },
  ) => {
    const currentPage = (search as SearchRecord)[pageKey]
    const pageNum = typeof currentPage === 'number' ? currentPage : defaultPage

    if (pageCount > 0 && pageNum > pageCount) {
      const patch: Record<string, unknown> = {
        [pageKey]: opts.resetTo === 'last' ? pageCount : undefined,
      }

      navigate({
        replace: true,
        search: buildQuery(search, patch),
      })
    }
  }

  return {
    globalFilter: globalFilterEnabled ? (globalFilter ?? '') : undefined,
    onGlobalFilterChange,
    columnFilters,
    onColumnFiltersChange,
    pagination,
    onPaginationChange,
    ensurePageInRange,
  }
}
