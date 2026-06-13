import { useMemo, useState } from 'react';



import { type ColumnDef, type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table';
import { Network, Shield, Workflow } from 'lucide-react';



import { AssignmentAccessFilter } from '@/components/assignment-access-filter';
import { AssignmentStats } from '@/components/assignment-stats';
import { Badge } from '@/components/badge';
import { Switch } from '@/components/switch';
import { type GroupRoleRow, useGroupRoleIds, useGroupRolesList, useManageGroupRoles } from '@/features/group/api/useGroupRoles';
import { GroupRolesBulkActions } from '@/features/group/components/GroupRolesBulkActions';
import { type GroupRoleFilter, groupRoleFilterOptions } from '@/features/group/model/groupRoleFilters';
import { Checkbox } from '@/shared/ui/checkbox';
import { DataTableColumnHeader } from '@/shared/ui/data-table';
import { DataTable } from '@/shared/ui/data-table/data-table';
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton';









































interface GroupRolesTabProps {
  groupId: string
}

export function GroupRolesTab({ groupId }: GroupRolesTabProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [roleFilter, setRoleFilter] = useState<GroupRoleFilter>('all')

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: 'name', desc: false }])

  const { data: rolesPage, isLoading: isRolesLoading } = useGroupRolesList(groupId, {
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
    filter: roleFilter,
  })

  const { data: directRoleIds = [], isLoading: isDirectRolesLoading } = useGroupRoleIds(
    groupId,
    'direct',
  )
  const { data: effectiveRoleIds = [], isLoading: isEffectiveRolesLoading } = useGroupRoleIds(
    groupId,
    'effective',
  )
  const { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation } =
    useManageGroupRoles(groupId)

  const isMutating =
    addMutation.isPending ||
    removeMutation.isPending ||
    bulkAddMutation.isPending ||
    bulkRemoveMutation.isPending

  const columns = useMemo<ColumnDef<GroupRoleRow>[]>(
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
        accessorKey: 'name',
        header: ({ column }) => <DataTableColumnHeader column={column} title="Role" />,
        cell: ({ row }) => (
          <div className="flex items-center gap-2 font-medium">
            <Shield className="text-muted-foreground h-4 w-4" />
            {row.getValue('name')}
          </div>
        ),
        enableSorting: true,
      },
      {
        accessorKey: 'description',
        header: 'Description',
        cell: ({ row }) => (
          <div className="text-muted-foreground max-w-[500px] truncate">
            {row.getValue('description') || '-'}
          </div>
        ),
      },
      {
        id: 'access',
        header: 'Access',
        cell: ({ row }) => {
          const role = row.original

          if (role.is_direct) {
            return <Badge variant="secondary">Direct</Badge>
          }

          if (role.is_effective) {
            return <Badge variant="outline">Composite</Badge>
          }

          return <span className="text-muted-foreground text-xs">—</span>
        },
        size: 140,
      },
      {
        id: 'direct',
        header: 'Direct',
        cell: ({ row }) => {
          const role = row.original
          const isDirect = role.is_direct

          return (
            <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
              <Switch
                checked={isDirect}
                disabled={isMutating}
                onCheckedChange={(checked) => {
                  if (checked) {
                    addMutation.mutate(role.id)
                  } else {
                    removeMutation.mutate(role.id)
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

  const handleFilterChange = (value: GroupRoleFilter) => {
    setRoleFilter(value)
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
  }

  return (
    <div className="flex h-full w-full flex-col gap-4">
      <AssignmentStats
        metrics={[
          { label: 'Direct', value: directRoleIds.length, icon: Shield },
          { label: 'Effective', value: effectiveRoleIds.length, icon: Network },
          { label: 'Roles', value: rolesPage?.meta.total ?? 0, icon: Workflow },
        ]}
      />

      {isRolesLoading || isDirectRolesLoading || isEffectiveRolesLoading ? (
        <div className="h-[calc(100vh-440px)]">
          <DataTableSkeleton columnCount={5} rowCount={10} />
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={rolesPage?.data || []}
          pageCount={rolesPage?.meta.total_pages || 0}
          pagination={pagination}
          onPaginationChange={handlePaginationChange}
          sorting={sorting}
          onSortingChange={handleSortingChange}
          searchKey="name"
          searchValue={searchTerm}
          onSearch={handleSearch}
          toolbarFilters={() => (
            <AssignmentAccessFilter
              options={groupRoleFilterOptions}
              value={roleFilter}
              onChange={handleFilterChange}
            />
          )}
          bulkEntityName="role"
          renderBulkActions={(table) => (
            <GroupRolesBulkActions
              selectedRoles={table.getFilteredSelectedRowModel().rows.map((row) => row.original)}
              isMutating={isMutating}
              onAssignRoles={(roleIds) =>
                bulkAddMutation.mutate(roleIds, { onSuccess: () => table.resetRowSelection() })
              }
              onRemoveRoles={(roleIds) =>
                bulkRemoveMutation.mutate(roleIds, { onSuccess: () => table.resetRowSelection() })
              }
            />
          )}
          className="max-h-[calc(100vh-590px)]"
        />
      )}
    </div>
  )
}
