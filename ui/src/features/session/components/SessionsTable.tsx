import { useMemo, useState } from 'react'

import { type OnChangeFn, type PaginationState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useSessionStore } from '@/entities/session/model/sessionStore.ts'
import type { Session } from '@/entities/session/model/types.ts'
import { useSessions } from '@/features/session/api/useSessions.ts'
import { RevokeOtherSessionsButton } from '@/features/session/components/RevokeOtherSessionsButton.tsx'
import { SessionBulkActions } from '@/features/session/components/SessionBulkActions.tsx'
import { SessionDetailsDrawer } from '@/features/session/components/SessionDetailsDrawer.tsx'
import { getSessionColumns } from '@/features/session/components/SessionColumns.tsx'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'

export function SessionsTable() {
  const [searchParams, setSearchParams] = useSearchParams()

  const { user } = useSessionStore()
  const currentSessionId = user?.sid

  const page = Number(searchParams.get('page')) || 1
  const perPage = Number(searchParams.get('per_page')) || 10
  const searchTerm = searchParams.get('q') || ''

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })

  const [detailsSession, setDetailsSession] = useState<Session | null>(null)
  const [detailsOpen, setDetailsOpen] = useState(false)

  const { data, isLoading } = useSessions({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    q: searchTerm,
  })

  const openDetails = (session: Session) => {
    setDetailsSession(session)
    setDetailsOpen(true)
  }

  const columns = useMemo(
    () => getSessionColumns(currentSessionId, openDetails),
    [currentSessionId],
  )

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
        <DataTableSkeleton columnCount={7} rowCount={10} />
      </div>
    )
  }

  return (
    <>
      <DataTable
        columns={columns}
        data={data?.data || []}
        pageCount={data?.meta.total_pages || 0}
        pagination={pagination}
        onPaginationChange={handlePaginationChange}
        searchKey="user_id"
        searchValue={searchTerm}
        onSearch={handleSearch}
        onRowClick={openDetails}
        customToolbarButtons={<RevokeOtherSessionsButton />}
        bulkEntityName="session"
        renderBulkActions={(table) => (
          <SessionBulkActions table={table} currentSessionId={currentSessionId} />
        )}
        className="max-h-[calc(100vh-328px)]"
      />
      <SessionDetailsDrawer
        session={detailsSession}
        currentSessionId={currentSessionId}
        open={detailsOpen}
        onOpenChange={setDetailsOpen}
      />
    </>
  )
}
