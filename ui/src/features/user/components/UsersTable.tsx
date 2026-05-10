import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useUsers } from '@/features/user/api/useUsers.ts'
import { userColumns } from '@/features/user/components/UserColumns.tsx'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton.tsx'
import { DataTable } from '@/shared/ui/data-table/data-table.tsx'
import { UsersPrimaryButtons } from '@/features/user/components/UsersPrimaryButtons.tsx'
import { UserBulkActions } from '@/features/user/components/UserBulkActions.tsx'
import { useDataTableUrlState } from '@/shared/lib/hooks/useDataTableUrlState'
import { type DataTableFilterField } from '@/shared/ui/data-table/types'

const userFilters: DataTableFilterField[] = [
  {
    key: 'email',
    label: 'Email',
    type: 'text',
    placeholder: 'Enter email...',
  },
  {
    key: 'created_at',
    label: 'Created',
    type: 'date-range',
  },
  {
    key: 'last_sign_in_at',
    label: 'Last signed in',
    type: 'date-range',
  },
]

export function UsersTable() {
  const navigate = useRealmNavigate()
  const {
    pagination,
    setPagination,
    sorting,
    setSorting,
    searchTerm,
    setSearchTerm,
    activeFilters,
    setActiveFilters,
  } = useDataTableUrlState('username', 'asc')

  const { data, isLoading } = useUsers({
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
    filters: activeFilters,
  })

  if (isLoading)
    return (
      <div className="h-[calc(100vh-240px)]">
        <DataTableSkeleton columnCount={5} rowCount={10} />
      </div>
    )

  return (
    <DataTable
      columns={userColumns}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={setPagination}
      sorting={sorting}
      onSortingChange={setSorting}
      searchKey="username"
      searchPlaceholder="Filter users..."
      searchValue={searchTerm}
      onSearch={setSearchTerm}
      onRowClick={(user) => navigate(`/users/${user.id}`)}
      customToolbarButtons={<UsersPrimaryButtons />}
      bulkEntityName="user"
      renderBulkActions={(table) => <UserBulkActions table={table} />}
      filters={userFilters}
      activeFilters={activeFilters}
      onFilterChange={setActiveFilters}
    />
  )
}
