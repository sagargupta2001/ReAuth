import { X } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import type { UserRoleRow } from '@/features/user/api/useUserRoles'

interface UserRolesBulkActionsProps {
  selectedRoles: UserRoleRow[]
  isMutating: boolean
  onAssignDirect: (roleIds: string[]) => void
  onRemoveDirect: (roleIds: string[]) => void
  onClearSelection: () => void
}

export function UserRolesBulkActions({
  selectedRoles,
  isMutating,
  onAssignDirect,
  onRemoveDirect,
  onClearSelection,
}: UserRolesBulkActionsProps) {
  if (!selectedRoles.length) return null

  const addIds = selectedRoles.filter((role) => !role.is_direct).map((role) => role.id)
  const removeIds = selectedRoles.filter((role) => role.is_direct).map((role) => role.id)

  return (
    <div
      role="toolbar"
      aria-label={`Bulk actions for ${selectedRoles.length} selected role${selectedRoles.length > 1 ? 's' : ''}`}
      className="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-xl transition-all delay-100 duration-300 ease-out hover:scale-105 focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:outline-none"
    >
      <div className="flex items-center gap-x-2 rounded-xl border bg-background/95 p-2 shadow-xl backdrop-blur-lg supports-[backdrop-filter]:bg-background/60">
        <Button
          size="icon"
          variant="outline"
          className="size-6 rounded-full"
          disabled={isMutating}
          onClick={onClearSelection}
          aria-label="Clear selection"
        >
          <X className="size-4" />
        </Button>

        <Separator className="h-5" orientation="vertical" aria-hidden="true" />

        <div className="flex items-center gap-x-1 text-sm">
          <Badge variant="default" className="min-w-8 justify-center rounded-lg">
            {selectedRoles.length}
          </Badge>
          <span className="hidden sm:inline">
            role{selectedRoles.length === 1 ? '' : 's'}
          </span>{' '}
          selected
        </div>

        <Separator className="h-5" orientation="vertical" aria-hidden="true" />

        <Button
          size="sm"
          variant="outline"
          disabled={addIds.length === 0 || isMutating}
          onClick={() => onAssignDirect(addIds)}
        >
          Assign Direct
        </Button>
        <Button
          size="sm"
          variant="destructive"
          disabled={removeIds.length === 0 || isMutating}
          onClick={() => onRemoveDirect(removeIds)}
        >
          Remove Direct
        </Button>
      </div>
    </div>
  )
}
