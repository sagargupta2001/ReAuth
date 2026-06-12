import { Button } from '@/components/button'
import type { RoleMemberRow } from '@/features/roles/api/useRoleMembers'

interface RoleMembersBulkActionsProps {
  selectedMembers: RoleMemberRow[]
  isMutating: boolean
  onAssignDirect: (userIds: string[]) => void
  onRemoveDirect: (userIds: string[]) => void
}

export function RoleMembersBulkActions({
  selectedMembers,
  isMutating,
  onAssignDirect,
  onRemoveDirect,
}: RoleMembersBulkActionsProps) {
  const assignableIds = selectedMembers.filter((member) => !member.is_direct).map((m) => m.id)
  const removableIds = selectedMembers.filter((member) => member.is_direct).map((m) => m.id)

  return (
    <>
      <Button
        size="sm"
        variant="outline"
        disabled={assignableIds.length === 0 || isMutating}
        onClick={() => onAssignDirect(assignableIds)}
      >
        Assign Direct
      </Button>
      <Button
        size="sm"
        variant="destructive"
        disabled={removableIds.length === 0 || isMutating}
        onClick={() => onRemoveDirect(removableIds)}
      >
        Remove Direct
      </Button>
    </>
  )
}
