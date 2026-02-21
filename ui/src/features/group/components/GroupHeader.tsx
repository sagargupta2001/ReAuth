import { useMemo, useState } from 'react'

import { Group as GroupIcon, MoreVertical, Trash2 } from 'lucide-react'
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Group } from '@/entities/group/model/types'
import { useDeleteGroup } from '@/features/group/api/useDeleteGroup'
import { useGroupDeleteSummary } from '@/features/group/api/useGroupDeleteSummary'
import { useGroupTreeStore } from '@/features/group-tree/model/groupTreeStore'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import { findNode, removeNode } from '@/features/group-tree/lib/tree-utils'
import { Checkbox } from '@/shared/ui/checkbox'

interface GroupHeaderProps {
  group: Group
  showBack?: boolean
}

const EMPTY_IDS: string[] = []

export function GroupHeader({ group, showBack = true }: GroupHeaderProps) {
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [cascade, setCascade] = useState(false)
  const updateTreeCache = useGroupTreeStore((state) => state.updateTree)
  const expandedByRealm = useGroupTreeStore((state) => state.expandedByRealm)
  const expandedIdsList = expandedByRealm[realm] ?? EMPTY_IDS
  const setExpanded = useGroupTreeStore((state) => state.setExpanded)
  const deleteGroup = useDeleteGroup(group.id)
  const { data: summary, isLoading: summaryLoading } = useGroupDeleteSummary(
    group.id,
    deleteOpen,
  )

  const hasDescendants = (summary?.descendant_count || 0) > 0
  const deleteDisabled = summaryLoading || (hasDescendants && !cascade)
  const impactLabel = useMemo(() => {
    if (!summary) return ''
    const totalGroups = summary.descendant_count + 1
    return `${totalGroups} group${totalGroups === 1 ? '' : 's'}`
  }, [summary])

  const copyId = () => {
    void navigator.clipboard.writeText(group.id)
    toast.success('Group ID copied')
  }

  const handleConfirmDelete = () => {
    deleteGroup.mutate(
      { cascade: hasDescendants ? cascade : false },
      {
        onSuccess: () => {
          let nextTree: GroupTreeNode[] = []
          updateTreeCache(realm, (prev) => {
            const result = removeNode(prev, group.id)
            nextTree = result.tree
            return result.tree
          })
          if (nextTree.length > 0) {
            const remaining = expandedIdsList.filter((id) => findNode(nextTree, id))
            setExpanded(realm, remaining)
          } else {
            setExpanded(realm, [])
          }
          setDeleteOpen(false)
          setCascade(false)
          navigate('/groups')
        },
      },
    )
  }

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center justify-between border-b px-6 backdrop-blur">
      <div className="flex flex-col gap-1">

        <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <GroupIcon className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{group.name}</h1>
          </div>
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            <span>ID:</span>
            <button onClick={copyId} className="hover:text-foreground font-mono hover:underline">
              {group.id}
            </button>
          </div>
        </div>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {showBack ? (
          <Button variant="outline" onClick={() => navigate('/groups')} size="sm">
            Back
          </Button>
        ) : null}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem className="text-destructive" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="mr-2 h-4 w-4" /> Delete Group
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <Dialog
        open={deleteOpen}
        onOpenChange={(open) => {
          setDeleteOpen(open)
          if (!open) {
            setCascade(false)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete group</DialogTitle>
            <DialogDescription>
              This permanently removes the group and unassigns any roles or members linked to it.
            </DialogDescription>
          </DialogHeader>

          {summaryLoading ? (
            <div className="text-muted-foreground text-sm">Loading impact...</div>
          ) : summary ? (
            <div className="space-y-3 text-sm">
              <div className="grid grid-cols-2 gap-2">
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Groups affected</div>
                  <div className="font-medium">{impactLabel}</div>
                </div>
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Sub-groups</div>
                  <div className="font-medium">{summary.descendant_count}</div>
                </div>
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Members affected</div>
                  <div className="font-medium">{summary.member_count}</div>
                </div>
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Roles affected</div>
                  <div className="font-medium">{summary.role_count}</div>
                </div>
              </div>

              {hasDescendants ? (
                <label className="flex items-start gap-3 rounded-md border p-3">
                  <Checkbox
                    checked={cascade}
                    onCheckedChange={(value) => setCascade(Boolean(value))}
                  />
                  <div className="space-y-1">
                    <div className="font-medium">
                      Also delete {summary.descendant_count} sub-group
                      {summary.descendant_count === 1 ? '' : 's'}
                    </div>
                    <div className="text-muted-foreground text-xs">
                      Required to remove nested groups inside this hierarchy.
                    </div>
                  </div>
                </label>
              ) : null}
            </div>
          ) : (
            <div className="text-destructive text-sm">Unable to load delete impact.</div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleteDisabled || deleteGroup.isPending || !summary}
            >
              {deleteGroup.isPending ? 'Deleting...' : 'Delete Group'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </header>
  )
}
