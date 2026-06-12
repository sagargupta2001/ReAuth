import { Button } from '@/components/button'
import type { RoleCompositeRow } from '@/features/roles/api/useRoleComposites'

interface RoleCompositesBulkActionsProps {
  selectedRoles: RoleCompositeRow[]
  isMutating: boolean
  onAddComposites: (roleIds: string[]) => void
  onRemoveComposites: (roleIds: string[]) => void
}

export function RoleCompositesBulkActions({
  selectedRoles,
  isMutating,
  onAddComposites,
  onRemoveComposites,
}: RoleCompositesBulkActionsProps) {
  const addIds = selectedRoles.filter((role) => !role.is_direct).map((role) => role.id)
  const removeIds = selectedRoles.filter((role) => role.is_direct).map((role) => role.id)

  return (
    <>
      <Button
        size="sm"
        variant="outline"
        disabled={addIds.length === 0 || isMutating}
        onClick={() => onAddComposites(addIds)}
      >
        Add Composites
      </Button>
      <Button
        size="sm"
        variant="destructive"
        disabled={removeIds.length === 0 || isMutating}
        onClick={() => onRemoveComposites(removeIds)}
      >
        Remove Composites
      </Button>
    </>
  )
}
