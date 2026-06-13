import { useMemo, useState } from 'react';



import { type ColumnDef, type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table';
import { UserCheck, UserCog, Users } from 'lucide-react';



import { Badge } from '@/components/badge';
import { Switch } from '@/components/switch';
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic';
import { type RoleMemberRow, useManageRoleMembers, useRoleMemberIds, useRoleMembersList } from '@/features/roles/api/useRoleMembers';
import { AssignmentAccessFilter } from '@/features/roles/components/AssignmentAccessFilter';
import { RoleAssignmentStats } from '@/features/roles/components/RoleAssignmentStats';
import { RoleMembersBulkActions } from '@/features/roles/components/RoleMembersBulkActions';
import { type RoleMemberFilter, roleMemberFilterOptions } from '@/features/roles/model/roleMemberFilters';
import { Checkbox } from '@/shared/ui/checkbox';
import { DataTableColumnHeader } from '@/shared/ui/data-table';
import { DataTable } from '@/shared/ui/data-table/data-table';
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton';























interface RoleMembersTabProps {
  roleId: string
}

export function RoleMembersTab({ roleId }: RoleMembersTabProps) {
  const navigate = useRealmNavigate()
  const [searchTerm, setSearchTerm] = useState('')
  const [assignmentFilter, setAssignmentFilter] = useState<RoleMemberFilter>('all')

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: 'username', desc: false }])

  const { data: directMemberIds = [] } = useRoleMemberIds(roleId, 'direct')
  const { data: effectiveMemberIds = [] } = useRoleMemberIds(roleId, 'effective')

  const {
    data: memberPage,
    isLoading: isMembersLoading,
  } = useRoleMembersList(roleId, {
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
    filter: assignmentFilter,
  })

  const { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation } =
    useManageRoleMembers(roleId)

  const isMutating =
    addMutation.isPending ||
    removeMutation.isPending ||
    bulkAddMutation.isPending ||
    bulkRemoveMutation.isPending

  const columns = useMemo<ColumnDef<RoleMemberRow>[]>(
    () => [
      {
        id: 'select',
        header: ({ table }) => (
          <div
            className="p-2"
            onClick={(e) => e.stopPropagation()}
            onMouseDown={(e) => e.stopPropagation()}
          >
            <Checkbox
              checked={
                table.getIsAllPageRowsSelected() ||
                (table.getIsSomePageRowsSelected() && 'indeterminate')
              }
              onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
              aria-label="Select all"
              className="translate-y-0.5"
            />
          </div>
        ),
        cell: ({ row }) => (
          <div
            className="p-2"
            onClick={(e) => e.stopPropagation()}
            onMouseDown={(e) => e.stopPropagation()}
          >
            <Checkbox
              checked={row.getIsSelected()}
              onCheckedChange={(value) => row.toggleSelected(!!value)}
              aria-label="Select row"
              className="translate-y-0.5"
            />
          </div>
        ),
        enableSorting: false,
        enableHiding: false,
        size: 40,
      },
      {
        accessorKey: 'username',
        header: ({ column }) => <DataTableColumnHeader column={column} title="Username" />,
        cell: ({ row }) => (
          <div className="flex items-center gap-2">
            <div className="bg-muted flex size-8 items-center justify-center rounded-full">
              <UserCog className="text-muted-foreground size-4" />
            </div>
            <span className="font-medium">{row.getValue('username')}</span>
          </div>
        ),
        enableSorting: true,
      },
      {
        accessorKey: 'id',
        header: ({ column }) => <DataTableColumnHeader column={column} title="User ID" />,
        cell: ({ row }) => (
          <div className="text-muted-foreground font-mono text-xs">{row.getValue('id')}</div>
        ),
        enableSorting: false,
      },
      {
        id: 'access',
        header: 'Access',
        cell: ({ row }) => {
          const user = row.original

          if (user.is_direct) {
            return <Badge variant="secondary">Direct</Badge>
          }

          if (user.is_effective) {
            return <Badge variant="outline">Via Group</Badge>
          }

          return <span className="text-muted-foreground text-xs">—</span>
        },
        size: 140,
      },
      {
        id: 'direct',
        header: 'Direct',
        cell: ({ row }) => {
          const user = row.original

          return (
            <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
              <Switch
                checked={user.is_direct}
                disabled={isMutating}
                onCheckedChange={(checked) => {
                  if (checked) {
                    addMutation.mutate(user.id)
                  } else {
                    removeMutation.mutate(user.id)
                  }
                }}
              />
            </div>
          )
        },
        size: 120,
      },
    ],
    [addMutation, isMutating, removeMutation],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(nextState)
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(sorting) : updater
    setSorting(nextState)
  }

  const handleSearch = (value: string) => {
    setSearchTerm(value)
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
  }

  const handleFilterChange = (value: RoleMemberFilter) => {
    setAssignmentFilter(value)
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
  }

  return (
    <div className="flex h-full w-full flex-col gap-4">
      <RoleAssignmentStats
        metrics={[
          { label: 'Direct', value: directMemberIds.length, icon: UserCheck },
          { label: 'Effective', value: effectiveMemberIds.length, icon: Users },
          { label: 'Total members', value: memberPage?.meta.total ?? 0, icon: UserCog },
        ]}
      />

      {isMembersLoading ? (
        <div className="h-[calc(100vh-440px)]">
          <DataTableSkeleton columnCount={5} rowCount={10} />
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={memberPage?.data || []}
          pageCount={memberPage?.meta.total_pages || 0}
          pagination={pagination}
          onPaginationChange={handlePaginationChange}
          sorting={sorting}
          onSortingChange={handleSortingChange}
          searchKey="username"
          searchValue={searchTerm}
          onSearch={handleSearch}
          toolbarFilters={() => (
            <AssignmentAccessFilter
              options={roleMemberFilterOptions}
              value={assignmentFilter}
              onChange={handleFilterChange}
            />
          )}
          onRowClick={(user) => navigate(`/users/${user.id}`)}
          bulkEntityName="user"
          renderBulkActions={(table) => (
            <RoleMembersBulkActions
              selectedMembers={table.getFilteredSelectedRowModel().rows.map((row) => row.original)}
              isMutating={isMutating}
              onAssignDirect={(userIds) =>
                bulkAddMutation.mutate(userIds, { onSuccess: () => table.resetRowSelection() })
              }
              onRemoveDirect={(userIds) =>
                bulkRemoveMutation.mutate(userIds, { onSuccess: () => table.resetRowSelection() })
              }
            />
          )}
        />
      )}
    </div>
  )
}
