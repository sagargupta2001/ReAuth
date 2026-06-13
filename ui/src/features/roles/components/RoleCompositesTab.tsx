import { useMemo, useState } from 'react';



import { type ColumnDef, type OnChangeFn, type PaginationState, type SortingState } from '@tanstack/react-table';
import { Layers, Network, Workflow } from 'lucide-react';



import { Badge } from '@/components/badge';
import { Switch } from '@/components/switch';
import { type RoleCompositeRow, useManageRoleComposites, useRoleCompositeIds, useRoleCompositesList } from '@/features/roles/api/useRoleComposites';
import { AssignmentAccessFilter } from '@/features/roles/components/AssignmentAccessFilter';
import { RoleAssignmentStats } from '@/features/roles/components/RoleAssignmentStats';
import { RoleCompositesBulkActions } from '@/features/roles/components/RoleCompositesBulkActions';
import { type RoleCompositeFilter, roleCompositeFilterOptions } from '@/features/roles/model/roleCompositeFilters';
import { Checkbox } from '@/shared/ui/checkbox';
import { DataTableColumnHeader } from '@/shared/ui/data-table';
import { DataTable } from '@/shared/ui/data-table/data-table';
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton';






















interface RoleCompositesTabProps {
  roleId: string
}

export function RoleCompositesTab({ roleId }: RoleCompositesTabProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [roleFilter, setRoleFilter] = useState<RoleCompositeFilter>('all')

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: 'name', desc: false }])

  const { data: rolesPage, isLoading: isRolesLoading } = useRoleCompositesList(roleId, {
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
    filter: roleFilter,
  })

  const { data: directRoleIds = [], isLoading: isDirectRolesLoading } = useRoleCompositeIds(
    roleId,
    'direct',
  )
  const { data: effectiveRoleIds = [], isLoading: isEffectiveRolesLoading } = useRoleCompositeIds(
    roleId,
    'effective',
  )

  const { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation } =
    useManageRoleComposites(roleId)

  const isMutating =
    addMutation.isPending ||
    removeMutation.isPending ||
    bulkAddMutation.isPending ||
    bulkRemoveMutation.isPending

  const columns = useMemo<ColumnDef<RoleCompositeRow>[]>(
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
            <Layers className="text-muted-foreground h-4 w-4" />
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
            return <Badge variant="outline">Inherited</Badge>
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

          return (
            <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
              <Switch
                checked={role.is_direct}
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

  const handleFilterChange = (value: RoleCompositeFilter) => {
    setRoleFilter(value)
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
  }

  return (
    <div className="flex h-full w-full flex-col gap-4">
      <RoleAssignmentStats
        metrics={[
          { label: 'Direct', value: directRoleIds.length, icon: Layers },
          { label: 'Effective', value: effectiveRoleIds.length, icon: Network },
          { label: 'Total roles', value: rolesPage?.meta.total ?? 0, icon: Workflow },
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
              options={roleCompositeFilterOptions}
              value={roleFilter}
              onChange={handleFilterChange}
            />
          )}
          bulkEntityName="role"
          renderBulkActions={(table) => (
            <RoleCompositesBulkActions
              selectedRoles={table.getFilteredSelectedRowModel().rows.map((row) => row.original)}
              isMutating={isMutating}
              onAddComposites={(roleIds) =>
                bulkAddMutation.mutate(roleIds, { onSuccess: () => table.resetRowSelection() })
              }
              onRemoveComposites={(roleIds) =>
                bulkRemoveMutation.mutate(roleIds, { onSuccess: () => table.resetRowSelection() })
              }
            />
          )}
        />
      )}
    </div>
  )
}
