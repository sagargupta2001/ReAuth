import { useMemo, useState } from 'react'

import {
  type ColumnDef,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
} from '@tanstack/react-table'
import { Layers } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Switch } from '@/components/switch'
import {
  type RoleCompositeRow,
  useManageRoleComposites,
  useRoleCompositeIds,
  useRoleCompositesList,
} from '@/features/roles/api/useRoleComposites'
import { DataTableColumnHeader } from '@/shared/ui/data-table'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'
import { Checkbox } from '@/shared/ui/checkbox'

interface RoleCompositesTabProps {
  roleId: string
}

type RoleFilter = 'all' | 'direct' | 'effective' | 'unassigned'

export function RoleCompositesTab({ roleId }: RoleCompositesTabProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [roleFilter, setRoleFilter] = useState<RoleFilter>('all')

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
          <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
            <Checkbox
              checked={
                table.getIsAllPageRowsSelected() ||
                (table.getIsSomePageRowsSelected() && 'indeterminate')
              }
              onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
              aria-label="Select all"
              className="translate-y-[2px]"
            />
          </div>
        ),
        cell: ({ row }) => (
          <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
            <Checkbox
              checked={row.getIsSelected()}
              onCheckedChange={(value) => row.toggleSelected(!!value)}
              aria-label="Select row"
              className="translate-y-[2px]"
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

  const filterOptions: { value: RoleFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'direct', label: 'Direct' },
    { value: 'effective', label: 'Inherited' },
    { value: 'unassigned', label: 'Unassigned' },
  ]

  return (
    <div className="flex h-full w-full flex-col gap-4">
      <div className="flex flex-col gap-3">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h3 className="text-lg font-semibold">Composite Roles</h3>
            <p className="text-muted-foreground text-sm">
              Build composite roles by combining existing roles.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline">Direct {directRoleIds.length}</Badge>
            <Badge variant="outline">Effective {effectiveRoleIds.length}</Badge>
            <Badge variant="outline">Roles {rolesPage?.meta.total ?? 0}</Badge>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {filterOptions.map((option) => (
            <Button
              key={option.value}
              size="sm"
              variant={roleFilter === option.value ? 'secondary' : 'outline'}
              onClick={() => {
                setRoleFilter(option.value)
                setPagination((prev) => ({ ...prev, pageIndex: 0 }))
              }}
            >
              {option.label}
            </Button>
          ))}
        </div>
      </div>

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
          searchPlaceholder="Filter roles..."
          searchValue={searchTerm}
          onSearch={handleSearch}
          bulkEntityName="role"
          renderBulkActions={(table) => {
            const selectedRoles = table.getFilteredSelectedRowModel().rows.map((row) => row.original)
            const addIds = selectedRoles.filter((role) => !role.is_direct).map((role) => role.id)
            const removeIds = selectedRoles.filter((role) => role.is_direct).map((role) => role.id)

            return (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={addIds.length === 0 || isMutating}
                  onClick={() =>
                    bulkAddMutation.mutate(addIds, {
                      onSuccess: () => table.resetRowSelection(),
                    })
                  }
                >
                  Add Composites
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  disabled={removeIds.length === 0 || isMutating}
                  onClick={() =>
                    bulkRemoveMutation.mutate(removeIds, {
                      onSuccess: () => table.resetRowSelection(),
                    })
                  }
                >
                  Remove Composites
                </Button>
              </>
            )
          }}
          className="h-[calc(100vh-450px)]"
        />
      )}
    </div>
  )
}
