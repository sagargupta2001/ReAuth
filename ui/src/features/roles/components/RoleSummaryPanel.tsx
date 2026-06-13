import { useState } from 'react'

import { format } from 'date-fns'
import { CalendarClock, Copy, Fingerprint, Trash2 } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useDeleteRole } from '@/features/roles/api/useDeleteRole'
import { useRoleDeleteSummary } from '@/features/roles/api/useRoleDeleteSummary'
import type { Role } from '@/features/roles/api/useRoles'

interface RoleSummaryPanelProps {
  role: Role
}

function formatDate(value?: string | null) {
  if (!value) return '-'
  return format(new Date(value), 'MMM d, yyyy, h:mm a')
}

function SummaryRow({
  icon: Icon,
  label,
  value,
  copyable = false,
}: {
  icon: typeof Fingerprint
  label: string
  value: string
  copyable?: boolean
}) {
  const canCopy = copyable && value !== '-'

  const copyValue = () => {
    if (!canCopy) return

    void navigator.clipboard
      .writeText(value)
      .then(() => toast.success(`${label} copied.`))
      .catch(() => toast.error(`Failed to copy ${label.toLowerCase()}.`))
  }

  return (
    <div className="border-border/60 border-b py-3 last:border-b-0">
      <div className="text-muted-foreground mb-1 flex items-center gap-2 text-xs font-medium">
        <Icon className="h-3.5 w-3.5" />
        {label}
      </div>
      <div className="flex min-w-0 items-center justify-between gap-2">
        <span className="truncate text-sm font-medium">{value}</span>
        {copyable ? (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 shrink-0"
            disabled={!canCopy}
            onClick={copyValue}
            aria-label={`Copy ${label}`}
          >
            <Copy className="h-3.5 w-3.5" />
          </Button>
        ) : null}
      </div>
    </div>
  )
}

export function RoleSummaryPanel({ role }: RoleSummaryPanelProps) {
  const navigate = useRealmNavigate()
  const [deleteOpen, setDeleteOpen] = useState(false)
  const deleteRole = useDeleteRole(role.id)
  const { data: summary, isLoading: summaryLoading } = useRoleDeleteSummary(
    role.id,
    deleteOpen,
  )

  const handleConfirmDelete = () => {
    deleteRole.mutate(undefined, {
      onSuccess: () => {
        setDeleteOpen(false)
        navigate('/roles')
      },
    })
  }

  return (
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="Role ID" value={role.id} copyable />
      <SummaryRow icon={CalendarClock} label="Role created on" value={formatDate(role.created_at)} />

      <button
        type="button"
        className="text-destructive hover:text-destructive/90 mt-4 flex items-center gap-2 text-sm font-medium"
        onClick={() => setDeleteOpen(true)}
      >
        <Trash2 className="h-4 w-4" />
        Delete role
      </button>

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent className="sm:max-w-[520px]">
          <DialogHeader className="px-6 pt-6">
            <DialogTitle>Delete role</DialogTitle>
            <DialogDescription>
              This permanently removes the role and clears assignments, composites, and permissions
              linked to it.
            </DialogDescription>
          </DialogHeader>

          <div className="px-6 pb-2">
            {summaryLoading ? (
              <div className="text-muted-foreground text-sm">Loading impact...</div>
            ) : summary ? (
              <div className="space-y-3 text-sm">
                <div className="grid grid-cols-2 gap-2">
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Direct users</div>
                    <div className="font-medium">{summary.direct_user_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Effective users</div>
                    <div className="font-medium">{summary.effective_user_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Groups assigned</div>
                    <div className="font-medium">{summary.group_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Parent composites</div>
                    <div className="font-medium">{summary.parent_role_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Child composites</div>
                    <div className="font-medium">{summary.child_role_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Permissions</div>
                    <div className="font-medium">{summary.permission_count}</div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-destructive text-sm">Unable to load delete impact.</div>
            )}
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={summaryLoading || deleteRole.isPending || !summary}
            >
              {deleteRole.isPending ? 'Deleting...' : 'Delete Role'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
