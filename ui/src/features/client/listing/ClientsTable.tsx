import { useEffect, useState } from 'react'

import { type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useClients } from '@/features/client/api/useClients.ts'
import { clientColumns } from '@/features/client/listing/components/ClientColumns.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'

export function ClientsTable() {
  const [searchParams, setSearchParams] = useSearchParams()
  const navigate = useRealmNavigate()

  // 1. URL Params
  const page = Number(searchParams.get('page')) || 1
  const perPage = Number(searchParams.get('per_page')) || 10
  const sortBy = searchParams.get('sort_by') || 'client_id'
  const sortDir = (searchParams.get('sort_dir') as 'asc' | 'desc') || 'asc'
  const queryFromUrl = searchParams.get('q') || ''

  // 2. Local Search State (for Debouncing)
  // We initialize it with the URL value so it persists on refresh
  const [searchTerm, setSearchTerm] = useState(queryFromUrl)

  // 3. React Table State
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: sortBy, desc: sortDir === 'desc' }])

  // DEBOUNCE EFFECT
  // Sync local searchTerm to URL after 500ms delay
  useEffect(() => {
    const timer = setTimeout(() => {
      // Only update URL if the value actually changed from what's in the URL
      if (searchTerm !== queryFromUrl) {
        const params = new URLSearchParams(searchParams)
        if (searchTerm) {
          params.set('q', searchTerm)
        } else {
          params.delete('q')
        }
        // Reset to page 1 on new search
        params.set('page', '1')
        setPagination((prev) => ({ ...prev, pageIndex: 0 }))
        setSearchParams(params)
      }
    }, 500)

    return () => clearTimeout(timer)
  }, [searchTerm, searchParams, setSearchParams, queryFromUrl])

  // Fetch Data (Uses URL param, not local state, to avoid rapid fetching)
  const { data, isLoading } = useClients({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: queryFromUrl, // Use the committed URL value
  })

  // Sync Pagination to URL
  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(nextState)

    const params = new URLSearchParams(searchParams)
    params.set('page', String(nextState.pageIndex + 1))
    params.set('per_page', String(nextState.pageSize))
    setSearchParams(params)
  }

  // Sync Sorting to URL
  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(sorting) : updater
    setSorting(nextState)

    const params = new URLSearchParams(searchParams)
    if (nextState.length > 0) {
      params.set('sort_by', nextState[0].id)
      params.set('sort_dir', nextState[0].desc ? 'desc' : 'asc')
    } else {
      params.delete('sort_by')
      params.delete('sort_dir')
    }
    setSearchParams(params)
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <DataTableSkeleton columnCount={4} rowCount={10} />
      </div>
    )
  }

  return (
    <DataTable
      onRowClick={(row) => navigate(`/clients/${row.id}`)}
      columns={clientColumns}
      data={data?.data || []}
      // Server-Side Props
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      // Search Config
      searchKey="client_id"
      searchPlaceholder="Filter by Client ID..."
      searchValue={searchTerm}
      onSearch={setSearchTerm}
      className="h-[calc(100vh-328px)]"
    />
  )
}
