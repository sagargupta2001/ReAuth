import { useState } from 'react'

import { useParams } from 'react-router-dom'

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Group } from '@/entities/group/model/types'
import { CreateGroupForm } from '@/features/group/forms/CreateGroupForm'
import { GroupTreePanel } from '@/features/group-tree/components/GroupTreePanel'
import {
  findNode,
  insertNode,
  sortTreeByName,
  updateNode,
} from '@/features/group-tree/lib/tree-utils'
import { useGroupTreeStore } from '@/features/group-tree/model/groupTreeStore'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import { Separator } from '@/shared/ui/separator'

const EMPTY_IDS: string[] = []

export function GroupsSidebar() {
  const { groupId } = useParams<{ groupId?: string }>()
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()
  const [createOpen, setCreateOpen] = useState(false)
  const [createParentId, setCreateParentId] = useState<string | null>(null)
  const updateTreeCache = useGroupTreeStore((state) => state.updateTree)
  const expandedByRealm = useGroupTreeStore((state) => state.expandedByRealm)
  const expandedIdsList = expandedByRealm[realm] ?? EMPTY_IDS

  const handleSelectGroup = (id: string) => {
    navigate(`/groups/${id}/settings`)
  }

  const handleCreateGroup = (parentId: string | null) => {
    setCreateParentId(parentId)
    setCreateOpen(true)
  }

  const handleCreateClose = () => {
    setCreateOpen(false)
    setCreateParentId(null)
  }

  const handleGroupCreated = (group: Group) => {
    const node: GroupTreeNode = {
      id: group.id,
      parent_id: group.parent_id ?? null,
      name: group.name,
      description: group.description ?? null,
      sort_order: group.sort_order ?? 0,
      has_children: false,
    }

    updateTreeCache(realm, (prev) => {
      if (!node.parent_id) {
        return sortTreeByName([...prev, node])
      }

      const parent = findNode(prev, node.parent_id)
      if (!parent) return prev

      if (parent.children || expandedIdsList.includes(node.parent_id)) {
        const insertIndex = parent.children ? parent.children.length : 0
        const next = insertNode(prev, node.parent_id, node, insertIndex)
        return sortTreeByName(next)
      }

      return updateNode(prev, node.parent_id, (existing) => ({
        ...existing,
        has_children: true,
      }))
    })
  }

  return (
    <div className="bg-muted/10 flex h-full w-full flex-col border-r">
      <div className="bg-background flex h-14 shrink-0 items-center justify-between px-4">
        <h2 className="truncate text-lg font-semibold tracking-tight">Group Explorer</h2>
      </div>

      <div className="min-h-0 flex-1 overflow-hidden">
        <GroupTreePanel
          selectedId={groupId}
          onSelect={handleSelectGroup}
          onCreateGroup={handleCreateGroup}
        />
      </div>

      <Dialog
        open={createOpen}
        onOpenChange={(open) => {
          if (!open) {
            handleCreateClose()
          } else {
            setCreateOpen(true)
          }
        }}
      >
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader className="pt-6 pl-6">
            <DialogTitle>{createParentId ? 'Create Sub-group' : 'Create Group'}</DialogTitle>
            <DialogDescription>
              {createParentId
                ? 'Add a new group under the selected parent.'
                : 'Create a new top-level group in this realm.'}
            </DialogDescription>
          </DialogHeader>

          <Separator className="my-1" />

          <CreateGroupForm
            isDialog
            parentId={createParentId}
            onCreated={handleGroupCreated}
            onSuccess={handleCreateClose}
            onCancel={handleCreateClose}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}
