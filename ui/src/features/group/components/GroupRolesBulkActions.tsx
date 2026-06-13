import { Button } from '@/components/button'
import type { GroupRoleRow } from '@/features/group/api/useGroupRoles'

interface GroupRolesBulkActionsProps {
  selectedRoles: GroupRoleRow[]
  isMutating: boolean
  onAssignRoles: (roleIds: string[]) => void
  onRemoveRoles: (roleIds: string[]) => void
}

export function GroupRolesBulkActions({
  selectedRoles,
  isMutating,
  onAssignRoles,
  onRemoveRoles,
}: GroupRolesBulkActionsProps) {
  const addIds = selectedRoles.filter((role) => !role.is_direct).map((role) => role.id)
  const removeIds = selectedRoles.filter((role) => role.is_direct).map((role) => role.id)

  return (
    <>
      <Button
        size="sm"
        variant="outline"
        disabled={addIds.length === 0 || isMutating}
        onClick={() => onAssignRoles(addIds)}
      >
        Assign Roles
      </Button>
      <Button
        size="sm"
        variant="destructive"
        disabled={removeIds.length === 0 || isMutating}
        onClick={() => onRemoveRoles(removeIds)}
      >
        Remove Roles
      </Button>
    </>
  )
}
