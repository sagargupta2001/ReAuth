import { useMemo, useState } from 'react'

import { useSessionStore } from '@/entities/session/model/sessionStore.ts'
import type { Session } from '@/entities/session/model/types.ts'
import { useSessions } from '@/features/session/api/useSessions.ts'
import { RevokeOtherSessionsButton } from '@/features/session/components/RevokeOtherSessionsButton.tsx'
import { SessionBulkActions } from '@/features/session/components/SessionBulkActions.tsx'
import { SessionDetailsDrawer } from '@/features/session/components/SessionDetailsDrawer.tsx'
import { getSessionColumns } from '@/features/session/components/SessionColumns.tsx'
import { useDataTableUrlState } from '@/shared/lib/hooks/useDataTableUrlState'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'
import { type DataTableFilterField } from '@/shared/ui/data-table/types'

const sessionFilters: DataTableFilterField[] = [
  {
    key: 'started',
    label: 'Started',
    type: 'date-range',
  },
]

export function SessionsTable() {
  const { user } = useSessionStore()
  const currentSessionId = user?.sid

  const { pagination, setPagination, searchTerm, setSearchTerm, activeFilters, setActiveFilters } =
    useDataTableUrlState('created_at', 'desc')

  const [detailsSession, setDetailsSession] = useState<Session | null>(null)
  const [detailsOpen, setDetailsOpen] = useState(false)

  const { data, isLoading } = useSessions({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    q: searchTerm,
    filters: activeFilters,
  })

  const openDetails = (session: Session) => {
    setDetailsSession(session)
    setDetailsOpen(true)
  }

  const columns = useMemo(
    () => getSessionColumns(currentSessionId, openDetails),
    [currentSessionId],
  )

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
        onPaginationChange={setPagination}
        searchKey="user_id"
        searchPlaceholder="Search by username or user ID..."
        searchValue={searchTerm}
        onSearch={setSearchTerm}
        onRowClick={openDetails}
        customToolbarButtons={<RevokeOtherSessionsButton />}
        bulkEntityName="session"
        renderBulkActions={(table) => (
          <SessionBulkActions table={table} currentSessionId={currentSessionId} />
        )}
        filters={sessionFilters}
        activeFilters={activeFilters}
        onFilterChange={setActiveFilters}
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
