import { type ChangeEvent, useMemo, useState } from 'react'

import { useSearchParams } from 'react-router-dom'

import type { PluginStatusInfo } from '@/entities/plugin/model/types'

export type PluginType = 'all' | 'active' | 'inactive'
export type SortType = 'asc' | 'desc'

export function usePluginFilters(plugins: PluginStatusInfo[] | undefined) {
  const [searchParams, setSearchParams] = useSearchParams()

  const [searchTerm, setSearchTerm] = useState(() => searchParams.get('filter') || '')
  const [appType, setAppType] = useState<PluginType>(
    () => (searchParams.get('type') as PluginType) || 'all',
  )
  const [sort, setSort] = useState<SortType>(() => (searchParams.get('sort') as SortType) || 'asc')

  const updateSearchParams = (updates: Record<string, string | undefined>) => {
    const params = new URLSearchParams(searchParams)
    Object.entries(updates).forEach(([key, value]) => {
      if (value === undefined) params.delete(key)
      else params.set(key, value)
    })
    setSearchParams(params)
  }

  const handleSearch = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setSearchTerm(value)
    updateSearchParams({ filter: value || undefined })
  }

  const handleTypeChange = (value: PluginType) => {
    setAppType(value)
    updateSearchParams({ type: value === 'all' ? undefined : value })
  }

  const handleSortChange = (value: SortType) => {
    setSort(value)
    updateSearchParams({ sort: value })
  }

  const filteredPlugins = useMemo(() => {
    if (!plugins) return []
    return plugins
      .sort((a, b) =>
        sort === 'asc'
          ? a.manifest.name.localeCompare(b.manifest.name)
          : b.manifest.name.localeCompare(a.manifest.name),
      )
      .filter((p) =>
        appType === 'active'
          ? p.status === 'active'
          : appType === 'inactive'
            ? p.status === 'inactive'
            : true,
      )
      .filter((p) => p.manifest.name.toLowerCase().includes(searchTerm.toLowerCase()))
  }, [plugins, sort, appType, searchTerm])

  return {
    searchTerm,
    appType,
    sort,
    filteredPlugins,
    handleSearch,
    handleTypeChange,
    handleSortChange,
  }
}
