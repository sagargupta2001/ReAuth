import { useState } from 'react'

import { type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

import { roleColumns } from './components/RoleColumns'
import { useRoles } from '@/entities/access/api/useRoles.ts'

export function RolesTable() {
  const navigate = useRealmNavigate()
  const [searchParams, setSearchParams] = useSearchParams()

  const page = Number(searchParams.get('page')) || 1
  const perPage = Number(searchParams.get('per_page')) || 10
  const sortBy = searchParams.get('sort_by') || 'name'
  const sortDir = (searchParams.get('sort_dir') as 'asc' | 'desc') || 'asc'
  const queryFromUrl = searchParams.get('q') || ''

  const [searchTerm, setSearchTerm] = useState(queryFromUrl)
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: sortBy, desc: sortDir === 'desc' }])

  const { data, isLoading } = useRoles({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
  })

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(nextState)
    const params = new URLSearchParams(searchParams)
    params.set('page', String(nextState.pageIndex + 1))
    params.set('per_page', String(nextState.pageSize))
    setSearchParams(params)
  }

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

  const handleSearch = (value: string) => {
    setSearchTerm(value)
    const params = new URLSearchParams(searchParams)
    if (value) {
      params.set('q', value)
      params.set('page', '1')
      setPagination((prev) => ({ ...prev, pageIndex: 0 }))
    } else {
      params.delete('q')
    }
    setSearchParams(params)
  }

  if (isLoading) {
    return (
      <div className="h-[calc(100vh-240px)]">
        <DataTableSkeleton columnCount={3} rowCount={5} />
      </div>
    )
  }

  return (
    <DataTable
      columns={roleColumns}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      searchKey="name"
      searchPlaceholder="Filter roles..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      onRowClick={(role) => navigate(`/access/roles/${role.id}`)}
      className="h-[calc(100vh-328px)]"
    />
  )
}
