import { useEffect, useMemo, useState } from 'react'
import type { OnChangeFn, PaginationState, SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { createPluginColumns, type PluginRow } from '@/features/events/components/PluginColumns'
import { usePluginMutations } from '@/features/plugin/api/usePluginMutations'
import { usePluginStatuses } from '@/features/plugin/api/usePluginStatuses'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

export function PluginsTable() {
  const navigate = useRealmNavigate()
  const [searchParams, setSearchParams] = useSearchParams()

  const pluginPage = Number(searchParams.get('plugin_page')) || 1
  const pluginPerPage = Number(searchParams.get('plugin_per_page')) || 10
  const pluginSortBy = searchParams.get('plugin_sort_by') || 'name'
  const pluginSortDir = (searchParams.get('plugin_sort_dir') as 'asc' | 'desc') || 'asc'
  const pluginQuery = searchParams.get('plugin_q') || ''

  const [pluginSearch, setPluginSearch] = useState(pluginQuery)

  const { data: pluginData, isLoading, isError } = usePluginStatuses({
    page: pluginPage,
    per_page: pluginPerPage,
    sort_by: pluginSortBy,
    sort_dir: pluginSortDir,
    q: pluginQuery || undefined,
  })
  const { enablePlugin, disablePlugin } = usePluginMutations()

  useEffect(() => {
    setPluginSearch(pluginQuery)
  }, [pluginQuery])

  useEffect(() => {
    const timer = setTimeout(() => {
      if (pluginSearch !== pluginQuery) {
        const params = new URLSearchParams(searchParams)
        if (pluginSearch) {
          params.set('plugin_q', pluginSearch)
        } else {
          params.delete('plugin_q')
        }
        params.set('plugin_page', '1')
        setSearchParams(params)
      }
    }, 400)
    return () => clearTimeout(timer)
  }, [pluginQuery, pluginSearch, searchParams, setSearchParams])

  const pluginRows = useMemo<PluginRow[]>(() => {
    const rows = pluginData?.data ?? []
    return rows.map((row) => ({
      id: row.manifest.id,
      name: row.manifest.name,
      version: row.manifest.version,
      status: row.status,
      events: row.manifest.events?.subscribes_to ?? [],
    }))
  }, [pluginData])

  const pagination = useMemo<PaginationState>(
    () => ({ pageIndex: pluginPage - 1, pageSize: pluginPerPage }),
    [pluginPage, pluginPerPage],
  )
  const sorting = useMemo<SortingState>(
    () => [{ id: pluginSortBy, desc: pluginSortDir === 'desc' }],
    [pluginSortBy, pluginSortDir],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination) : updater
    const params = new URLSearchParams(searchParams)
    params.set('plugin_page', String(next.pageIndex + 1))
    params.set('plugin_per_page', String(next.pageSize))
    setSearchParams(params)
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const next = typeof updater === 'function' ? updater(sorting) : updater
    const params = new URLSearchParams(searchParams)
    if (next.length) {
      params.set('plugin_sort_by', next[0].id)
      params.set('plugin_sort_dir', next[0].desc ? 'desc' : 'asc')
    } else {
      params.delete('plugin_sort_by')
      params.delete('plugin_sort_dir')
    }
    params.set('plugin_page', '1')
    setSearchParams(params)
  }

  const pluginColumns = useMemo(
    () =>
      createPluginColumns({
        onToggle: (id, enabled) =>
          enabled ? enablePlugin.mutate(id) : disablePlugin.mutate(id),
        isPending: enablePlugin.isPending || disablePlugin.isPending,
      }),
    [disablePlugin, enablePlugin],
  )

  if (isLoading) {
    return <DataTableSkeleton columnCount={4} rowCount={6} />
  }

  if (isError) {
    return (
      <div className="py-6 text-center text-sm text-muted-foreground">
        Failed to load plugin registry.
      </div>
    )
  }

  return (
    <DataTable<PluginRow, unknown>
      columns={pluginColumns}
      data={pluginRows}
      pageCount={pluginData?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      searchPlaceholder="Search plugins..."
      searchValue={pluginSearch}
      onSearch={setPluginSearch}
      className="h-[520px]"
      onRowClick={(row) => navigate(`/events/plugins/${row.id}`)}
    />
  )
}
