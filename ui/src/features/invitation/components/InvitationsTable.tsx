import { useMemo, useState } from 'react'

import { type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useInvitations, useResendInvitation, useRevokeInvitation } from '@/features/invitation/api/useInvitations'
import { getInvitationColumns } from '@/features/invitation/components/InvitationColumns'
import { UsersPrimaryButtons } from '@/features/user/components/UsersPrimaryButtons'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

export function InvitationsTable() {
  const [searchParams, setSearchParams] = useSearchParams()

  const page = Number(searchParams.get('inv_page')) || 1
  const perPage = Number(searchParams.get('inv_per_page')) || 10
  const sortBy = searchParams.get('inv_sort_by') || 'created_at'
  const sortDir = (searchParams.get('inv_sort_dir') as 'asc' | 'desc') || 'desc'
  const queryFromUrl = searchParams.get('inv_q') || ''

  const [searchTerm, setSearchTerm] = useState(queryFromUrl)
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: page - 1,
    pageSize: perPage,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: sortBy, desc: sortDir === 'desc' }])

  const { data, isLoading } = useInvitations({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm || undefined,
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
      searchKey="email"
      searchPlaceholder="Filter invitations..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      customToolbarButtons={<UsersPrimaryButtons />}
    />
  )
}
