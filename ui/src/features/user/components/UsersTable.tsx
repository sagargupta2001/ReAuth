import { useState } from 'react'

import { type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useUsers } from '@/features/user/api/useUsers.ts'
import { userColumns } from '@/features/user/components/UserColumns.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'

export function UsersTable() {
  const navigate = useRealmNavigate()
  const [searchParams, setSearchParams] = useSearchParams()

  // Initialize State from URL
  const page = Number(searchParams.get('page')) || 1
  const perPage = Number(searchParams.get('per_page')) || 10
  const sortBy = searchParams.get('sort_by') || 'username'
  const sortDir = (searchParams.get('sort_dir') as 'asc' | 'desc') || 'asc'
  const queryFromUrl = searchParams.get('q') || ''

  const [searchTerm, setSearchTerm] = useState(queryFromUrl)

  // React Table State
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: sortBy, desc: sortDir === 'desc' }])

  // 4. Fetch Data
  const { data, isLoading } = useUsers({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm, // Pass local term if debouncing is handled in DataTable or here
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

  // Sync Search to URL (Simple implementation)
  const handleSearch = (value: string) => {
    setSearchTerm(value)
    const params = new URLSearchParams(searchParams)
    if (value) {
      params.set('q', value)
      params.set('page', '1') // Reset page on search
      setPagination((prev) => ({ ...prev, pageIndex: 0 }))
    } else {
      params.delete('q')
    }
    setSearchParams(params)
  }

  if (isLoading) {
    return (
      <div className="h-[calc(100vh-240px)]">
        <DataTableSkeleton columnCount={4} rowCount={10} />
      </div>
    )
  }

  return (
    <DataTable
      columns={userColumns}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      // State Passing
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      // Search
      searchKey="username"
      searchPlaceholder="Filter users..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      // Row Click -> Edit Page
      onRowClick={(user) => navigate(`/users/${user.id}`)}
      // Layout
      className="h-[calc(100vh-328px)]"
    />
  )
}
