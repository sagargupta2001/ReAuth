import { Button } from '@/components/button'
import type { GroupMemberRow } from '@/features/group/api/useGroupMembers'

interface GroupMembersBulkActionsProps {
  selectedMembers: GroupMemberRow[]
  isMutating: boolean
  onAddToGroup: (userIds: string[]) => void
  onRemoveFromGroup: (userIds: string[]) => void
}

export function GroupMembersBulkActions({
  selectedMembers,
  isMutating,
  onAddToGroup,
  onRemoveFromGroup,
}: GroupMembersBulkActionsProps) {
  const addIds = selectedMembers.filter((member) => !member.is_member).map((m) => m.id)
  const removeIds = selectedMembers.filter((member) => member.is_member).map((m) => m.id)

  return (
    <>
      <Button
        size="sm"
        variant="outline"
        disabled={addIds.length === 0 || isMutating}
        onClick={() => onAddToGroup(addIds)}
      >
        Add to Group
      </Button>
      <Button
        size="sm"
        variant="destructive"
        disabled={removeIds.length === 0 || isMutating}
        onClick={() => onRemoveFromGroup(removeIds)}
      >
        Remove from Group
      </Button>
    </>
  )
}
