import { useMemo, useState } from 'react'

import {
  type ColumnFiltersState,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
} from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { invitationStatuses, type InvitationStatus } from '@/entities/invitation/model/types'
import {
  useInvitations,
  useResendInvitation,
  useRevokeInvitation,
} from '@/features/invitation/api/useInvitations'
import { getInvitationColumns } from '@/features/invitation/components/InvitationColumns'
import { UsersPrimaryButtons } from '@/features/user/components/UsersPrimaryButtons'
import { DataTableFacetedFilter } from '@/shared/ui/data-table'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

const invitationStatusFilterOptions = [
  { value: 'pending', label: 'Pending' },
  { value: 'accepted', label: 'Accepted' },
  { value: 'expired', label: 'Expired' },
  { value: 'revoked', label: 'Revoked' },
] satisfies Array<{ value: InvitationStatus; label: string }>

function parseStatusParam(value: string | null): InvitationStatus[] {
  if (!value) return []
  const allowed = new Set<string>(invitationStatuses)
  return value
    .split(',')
    .map((item) => item.trim())
    .filter((item): item is InvitationStatus => allowed.has(item))
}

function getStatusFilterValue(columnFilters: ColumnFiltersState): InvitationStatus[] {
  const value = columnFilters.find((filter) => filter.id === 'status')?.value
  if (!Array.isArray(value)) return []
  const allowed = new Set<string>(invitationStatuses)
  return value.filter(
    (item): item is InvitationStatus => typeof item === 'string' && allowed.has(item),
  )
}

export function InvitationsTable() {
  const [searchParams, setSearchParams] = useSearchParams()

  const page = Number(searchParams.get('inv_page')) || 1
  const perPage = Number(searchParams.get('inv_per_page')) || 10
  const sortBy = searchParams.get('inv_sort_by') || 'created_at'
  const sortDir = (searchParams.get('inv_sort_dir') as 'asc' | 'desc') || 'desc'
  const queryFromUrl = searchParams.get('inv_q') || ''
  const statusFromUrl = parseStatusParam(searchParams.get('inv_status'))

  const [searchTerm, setSearchTerm] = useState(queryFromUrl)
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: sortBy, desc: sortDir === 'desc' }])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>(
    statusFromUrl.length ? [{ id: 'status', value: statusFromUrl }] : [],
  )

  const statusFilter = getStatusFilterValue(columnFilters)

  const { data, isLoading } = useInvitations({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm || undefined,
    status: statusFilter,
  })

  const resendMutation = useResendInvitation()
  const revokeMutation = useRevokeInvitation()

  const columns = useMemo(
    () =>
      getInvitationColumns({
        onResend: (id) => resendMutation.mutate(id),
        onRevoke: (id) => revokeMutation.mutate(id),
        actionsDisabled: resendMutation.isPending || revokeMutation.isPending,
      }),
    [resendMutation, revokeMutation],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(next)

    const params = new URLSearchParams(searchParams)
    params.set('inv_page', String(next.pageIndex + 1))
    params.set('inv_per_page', String(next.pageSize))
    setSearchParams(params)
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const next = typeof updater === 'function' ? updater(sorting) : updater
    setSorting(next)

    const params = new URLSearchParams(searchParams)
    if (next.length) {
      params.set('inv_sort_by', next[0].id)
      params.set('inv_sort_dir', next[0].desc ? 'desc' : 'asc')
    } else {
      params.delete('inv_sort_by')
      params.delete('inv_sort_dir')
    }
    setSearchParams(params)
  }

  const handleSearch = (value: string) => {
    setSearchTerm(value)
    const params = new URLSearchParams(searchParams)
    if (value) {
      params.set('inv_q', value)
      params.set('inv_page', '1')
      setPagination((prev) => ({ ...prev, pageIndex: 0 }))
    } else {
      params.delete('inv_q')
    }
    setSearchParams(params)
  }

  const handleColumnFiltersChange: OnChangeFn<ColumnFiltersState> = (updater) => {
    const next = typeof updater === 'function' ? updater(columnFilters) : updater
    setColumnFilters(next)

    const statuses = getStatusFilterValue(next)
    const params = new URLSearchParams(searchParams)
    if (statuses.length) {
      params.set('inv_status', statuses.join(','))
    } else {
      params.delete('inv_status')
    }
    params.set('inv_page', '1')
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
    setSearchParams(params)
  }

  if (isLoading) {
    return (
      <div className="h-[calc(100vh-240px)]">
        <DataTableSkeleton columnCount={7} rowCount={10} />
      </div>
    )
  }

  return (
    <DataTable
      columns={columns}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      columnFilters={columnFilters}
      onColumnFiltersChange={handleColumnFiltersChange}
      searchKey="email"
      searchPlaceholder="Filter invitations..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      toolbarFilters={(table) => (
        <DataTableFacetedFilter
          column={table.getColumn('status')}
          title="Status"
          options={invitationStatusFilterOptions}
        />
      )}
      customToolbarButtons={<UsersPrimaryButtons />}
    />
  )
}
