import { useState } from 'react'

import { type OnChangeFn, type PaginationState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useSessionStore } from '@/entities/session/model/sessionStore.ts'
import { useRevokeSession, useSessions } from '@/features/session/api/useSessions.ts'
import { getSessionColumns } from '@/features/session/components/SessionColumns.tsx'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'

export function SessionsTable() {
  const [searchParams, setSearchParams] = useSearchParams()

  // 1. Get Current Session ID from Store
  const { user } = useSessionStore()
  const currentSessionId = user?.sid

  // 2. State
  const page = Number(searchParams.get('page')) || 1
  const perPage = Number(searchParams.get('per_page')) || 10
  const searchTerm = searchParams.get('q') || ''

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })

  // 3. API Hooks
  const { data, isLoading } = useSessions({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    q: searchTerm,
  })

  const revokeMutation = useRevokeSession()

  // 4. Handlers
  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(nextState)
    const params = new URLSearchParams(searchParams)
    params.set('page', String(nextState.pageIndex + 1))
    params.set('per_page', String(nextState.pageSize))
    setSearchParams(params)
  }

  const handleSearch = (value: string) => {
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
      <div className="h-[calc(100vh-200px)]">
        <DataTableSkeleton columnCount={6} rowCount={10} />
      </div>
    )
  }

  return (
    <DataTable
      columns={getSessionColumns(currentSessionId, (id) => revokeMutation.mutate(id))}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      searchKey="user_id"
      searchPlaceholder="Search by User ID..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      className="h-[calc(100vh-328px)]"
    />
  )
}
