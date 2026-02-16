import { useMemo, useState } from 'react'

import {
  type ColumnDef,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
} from '@tanstack/react-table'
import { UserCog } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Switch } from '@/components/switch'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import {
  useGroupMemberIds,
  useGroupMembersList,
  useManageGroupMembers,
  type GroupMemberRow,
} from '@/features/group/api/useGroupMembers'
import { DataTableColumnHeader } from '@/shared/ui/data-table'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'
import { Checkbox } from '@/shared/ui/checkbox'

interface GroupMembersTabProps {
  groupId: string
}

type MemberFilter = 'all' | 'members' | 'non-members'

export function GroupMembersTab({ groupId }: GroupMembersTabProps) {
  const navigate = useRealmNavigate()
  const [searchTerm, setSearchTerm] = useState('')
  const [memberFilter, setMemberFilter] = useState<MemberFilter>('all')

  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: 'username', desc: false }])

  const { data: memberIds = [] } = useGroupMemberIds(groupId)
  const { data: membersPage, isLoading: isMembersLoading } = useGroupMembersList(groupId, {
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
    filter: memberFilter,
  })

  const { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation } =
    useManageGroupMembers(groupId)

  const isMutating =
    addMutation.isPending ||
    removeMutation.isPending ||
    bulkAddMutation.isPending ||
    bulkRemoveMutation.isPending

  const columns = useMemo<ColumnDef<GroupMemberRow>[]>(
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
        id: 'member',
        header: 'Member',
        cell: ({ row }) => {
          const user = row.original

          return (
            <div onClick={(e) => e.stopPropagation()} onMouseDown={(e) => e.stopPropagation()}>
              <Switch
                checked={user.is_member}
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

  const filterOptions: { value: MemberFilter; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'members', label: 'Members' },
    { value: 'non-members', label: 'Not Members' },
  ]

  return (
    <div className="flex h-full w-full flex-col gap-4">
      <div className="flex flex-col gap-3">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h3 className="text-lg font-semibold">Members</h3>
            <p className="text-muted-foreground text-sm">Assign users to this group.</p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="secondary">Members {memberIds.length}</Badge>
            <Badge variant="outline">Users {membersPage?.meta.total ?? 0}</Badge>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {filterOptions.map((option) => (
            <Button
              key={option.value}
              size="sm"
              variant={memberFilter === option.value ? 'secondary' : 'outline'}
              onClick={() => {
                setMemberFilter(option.value)
                setPagination((prev) => ({ ...prev, pageIndex: 0 }))
              }}
            >
              {option.label}
            </Button>
          ))}
        </div>
      </div>

      {isMembersLoading ? (
        <div className="h-[calc(100vh-440px)]">
          <DataTableSkeleton columnCount={4} rowCount={10} />
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={membersPage?.data || []}
          pageCount={membersPage?.meta.total_pages || 0}
          pagination={pagination}
          onPaginationChange={handlePaginationChange}
          sorting={sorting}
          onSortingChange={handleSortingChange}
          searchKey="username"
          searchPlaceholder="Filter users..."
          searchValue={searchTerm}
          onSearch={handleSearch}
          onRowClick={(user) => navigate(`/users/${user.id}`)}
          bulkEntityName="user"
          renderBulkActions={(table) => {
            const selectedUsers = table.getFilteredSelectedRowModel().rows.map((row) => row.original)
            const addIds = selectedUsers.filter((user) => !user.is_member).map((user) => user.id)
            const removeIds = selectedUsers.filter((user) => user.is_member).map((user) => user.id)

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
                  Add to Group
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
                  Remove from Group
                </Button>
              </>
            )
          }}
          className="h-[calc(100vh-590px)]"
        />
      )}
    </div>
  )
}
