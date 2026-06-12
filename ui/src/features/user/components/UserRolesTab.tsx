import { useEffect, useMemo, useState } from 'react'

import {
  type UserRoleFilter,
  type UserRoleRow,
  useInfiniteUserRolesList,
  useManageUserRoles,
} from '@/features/user/api/useUserRoles'
import { UserRolesBulkActions } from '@/features/user/components/UserRolesBulkActions'
import { UserRolesToolbar } from '@/features/user/components/UserRolesToolbar'
import { UserRolesVirtualList } from '@/features/user/components/UserRolesVirtualList'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

interface UserRolesTabProps {
  userId: string
}

export function UserRolesTab({ userId }: UserRolesTabProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [roleFilter, setRoleFilter] = useState<UserRoleFilter>('all')
  const [selectedRoleIds, setSelectedRoleIds] = useState<Set<string>>(() => new Set())

  const roleListParams = useMemo(
    () => ({
      per_page: 25,
      sort_by: 'name',
      sort_dir: 'asc' as const,
      q: searchTerm || undefined,
      filter: roleFilter,
    }),
    [roleFilter, searchTerm],
  )

  const {
    data: rolesPage,
    isLoading: isRolesLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteUserRolesList(userId, roleListParams)

  const roles = useMemo(
    () => rolesPage?.pages.flatMap((page) => page.data) ?? [],
    [rolesPage?.pages],
  )

  const { addMutation, removeMutation, bulkAddMutation, bulkRemoveMutation } =
    useManageUserRoles(userId)

  const isMutating =
    addMutation.isPending ||
    removeMutation.isPending ||
    bulkAddMutation.isPending ||
    bulkRemoveMutation.isPending

  const selectedRoles = useMemo(
    () => roles.filter((role) => selectedRoleIds.has(role.id)),
    [roles, selectedRoleIds],
  )

  useEffect(() => {
    setSelectedRoleIds(new Set())
  }, [roleFilter, searchTerm, userId])

  const handleToggleRoleSelection = (roleId: string, selected: boolean) => {
    setSelectedRoleIds((current) => {
      const next = new Set(current)
      if (selected) {
        next.add(roleId)
      } else {
        next.delete(roleId)
      }
      return next
    })
  }

  const handleToggleLoadedSelection = (selected: boolean) => {
    setSelectedRoleIds((current) => {
      if (!selected) return new Set()
      const next = new Set(current)
      roles.forEach((role) => next.add(role.id))
      return next
    })
  }

  const handleSetDirect = (role: UserRoleRow, checked: boolean) => {
    if (checked) {
      addMutation.mutate(role.id)
    } else {
      removeMutation.mutate(role.id)
    }
  }

  const handleAssignDirect = (roleIds: string[]) => {
    bulkAddMutation.mutate(roleIds, {
      onSuccess: () => setSelectedRoleIds(new Set()),
    })
  }

  const handleRemoveDirect = (roleIds: string[]) => {
    bulkRemoveMutation.mutate(roleIds, {
      onSuccess: () => setSelectedRoleIds(new Set()),
    })
  }

  return (
    <div className="flex h-full w-full min-w-0 flex-col gap-4">
      <UserRolesToolbar
        searchValue={searchTerm}
        onSearchChange={setSearchTerm}
        filterValue={roleFilter}
        onFilterChange={setRoleFilter}
      />

      <UserRolesBulkActions
        selectedRoles={selectedRoles}
        isMutating={isMutating}
        onAssignDirect={handleAssignDirect}
        onRemoveDirect={handleRemoveDirect}
        onClearSelection={() => setSelectedRoleIds(new Set())}
      />

      {isRolesLoading ? (
        <div className="h-[calc(100vh-440px)]">
          <DataTableSkeleton columnCount={4} rowCount={10} />
        </div>
      ) : (
        <UserRolesVirtualList
          roles={roles}
          hasNextPage={hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          isMutating={isMutating}
          selectedRoleIds={selectedRoleIds}
          onFetchNextPage={() => {
            void fetchNextPage()
          }}
          onToggleRoleSelection={handleToggleRoleSelection}
          onToggleLoadedSelection={handleToggleLoadedSelection}
          onSetDirect={handleSetDirect}
        />
      )}
    </div>
  )
}
