import { useCallback, useEffect, useMemo, useState } from 'react'

import {
  DndContext,
  DragOverlay,
  type DragEndEvent,
  type DragStartEvent,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { SortableContext, arrayMove, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { Loader2, Search } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import {
  fetchGroupChildren,
  fetchGroupRoots,
  moveGroup,
} from '@/features/group-tree/api/groupTreeApi'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import {
  findNode,
  findPath,
  flattenTree,
  insertNode,
  removeChildrenOf,
  removeNode,
  reorderChildren,
  updateNode,
} from '@/features/group-tree/lib/tree-utils'
import { GroupTreeItem } from '@/features/group-tree/components/GroupTreeItem'
import { cn } from '@/lib/utils'

interface GroupTreePanelProps {
  selectedId?: string
  onSelect: (groupId: string) => void
  onCreateGroup: (parentId: string | null) => void
  refreshKey?: number
  onPathChange?: (path: GroupTreeNode[]) => void
}

export function GroupTreePanel({
  selectedId,
  onSelect,
  onCreateGroup,
  refreshKey,
  onPathChange,
}: GroupTreePanelProps) {
  const realm = useActiveRealm()
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }))

  const [tree, setTree] = useState<GroupTreeNode[]>([])
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set())
  const [loading, setLoading] = useState(false)
  const [loadingIds, setLoadingIds] = useState<Set<string>>(new Set())
  const [search, setSearch] = useState('')
  const [activeId, setActiveId] = useState<string | null>(null)

  const loadRoots = useCallback(async () => {
    setLoading(true)
    try {
      const response = await fetchGroupRoots(realm, {
        page: 1,
        per_page: 200,
        sort_by: 'sort_order',
        sort_dir: 'asc',
        q: search.trim() ? search.trim() : undefined,
      })

      setTree(response.data)
      setExpandedIds(new Set())
    } catch (err: any) {
      toast.error(err.message || 'Failed to load groups')
    } finally {
      setLoading(false)
    }
  }, [realm, search])

  useEffect(() => {
    void loadRoots()
  }, [loadRoots, refreshKey])

  useEffect(() => {
    if (!selectedId) {
      onPathChange?.([])
      return
    }
    const path = findPath(tree, selectedId)
    onPathChange?.(path ?? [])
  }, [onPathChange, selectedId, tree])

  const loadChildren = useCallback(
    async (groupId: string) => {
      if (loadingIds.has(groupId)) return
      setLoadingIds((prev) => new Set(prev).add(groupId))

      try {
        const response = await fetchGroupChildren(realm, groupId, {
          page: 1,
          per_page: 200,
          sort_by: 'sort_order',
          sort_dir: 'asc',
        })

        setTree((prev) =>
          updateNode(prev, groupId, (node) => ({
            ...node,
            children: response.data,
            has_children: response.meta.total > 0,
          })),
        )
        setExpandedIds((prev) => new Set(prev).add(groupId))
      } catch (err: any) {
        toast.error(err.message || 'Failed to load group children')
      } finally {
        setLoadingIds((prev) => {
          const next = new Set(prev)
          next.delete(groupId)
          return next
        })
      }
    },
    [loadingIds, realm],
  )

  const toggleExpand = useCallback(
    (groupId: string) => {
      const isExpanded = expandedIds.has(groupId)
      if (isExpanded) {
        setExpandedIds((prev) => {
          const next = new Set(prev)
          next.delete(groupId)
          return next
        })
        return
      }

      const node = findNode(tree, groupId)
      if (!node) return

      if (!node.children && node.has_children) {
        void loadChildren(groupId)
        return
      }

      setExpandedIds((prev) => new Set(prev).add(groupId))
    },
    [expandedIds, loadChildren, tree],
  )

  const flattenedTree = useMemo(() => flattenTree(tree), [tree])

  const collapsedIds = useMemo(() => {
    const ids = new Set<string>()
    flattenedTree.forEach((item) => {
      if (item.has_children && !expandedIds.has(item.id)) {
        ids.add(item.id)
      }
    })
    if (activeId) ids.add(activeId)
    return ids
  }, [expandedIds, flattenedTree, activeId])

  const visibleItems = useMemo(
    () => removeChildrenOf(flattenedTree, collapsedIds),
    [collapsedIds, flattenedTree],
  )

  const activeItem = useMemo(
    () => visibleItems.find((item) => item.id === activeId) ?? null,
    [activeId, visibleItems],
  )

  const handleDragStart = ({ active }: DragStartEvent) => {
    setActiveId(active.id as string)
  }

  const handleDragEnd = async ({ active, over }: DragEndEvent) => {
    if (!over) {
      setActiveId(null)
      return
    }

    const activeId = active.id as string
    const overId = over.id as string
    if (activeId === overId) {
      setActiveId(null)
      return
    }

    const overItem = visibleItems.find((item) => item.id === overId)
    if (!overItem) {
      setActiveId(null)
      return
    }

    const activeRect =
      active.rect.current?.translated || active.rect.current?.initial || over.rect
    const overRect = over.rect
    const centerY = activeRect.top + activeRect.height / 2
    const upper = overRect.top + overRect.height * 0.25
    const lower = overRect.top + overRect.height * 0.75
    const dropOnNode = centerY > upper && centerY < lower

    if (dropOnNode) {
      const targetParentId = overId
      const parentNode = findNode(tree, targetParentId)
      const insertIndex = parentNode?.children ? parentNode.children.length : 0

      setTree((prev) => {
        const { node, tree: removedTree } = removeNode(prev, activeId)
        if (!node) return prev

        const updatedNode = { ...node, parent_id: targetParentId }

        let nextTree = removedTree
        if (parentNode?.children) {
          nextTree = insertNode(nextTree, targetParentId, updatedNode, insertIndex)
          nextTree = updateNode(nextTree, targetParentId, (parent) => ({
            ...parent,
            has_children: true,
          }))
        } else {
          nextTree = updateNode(nextTree, targetParentId, (parent) => ({
            ...parent,
            has_children: true,
          }))
        }

        if (node.parent_id && node.parent_id !== targetParentId) {
          const oldParent = findNode(nextTree, node.parent_id)
          if (oldParent && (!oldParent.children || oldParent.children.length === 0)) {
            nextTree = updateNode(nextTree, node.parent_id, (parent) => ({
              ...parent,
              has_children: false,
            }))
          }
        }

        return nextTree
      })

      try {
        await moveGroup(realm, activeId, {
          parent_id: targetParentId,
        })
        if (!parentNode?.children) {
          void loadChildren(targetParentId)
        } else {
          setExpandedIds((prev) => new Set(prev).add(targetParentId))
        }
      } catch (err: any) {
        toast.error(err.message || 'Failed to move group')
        void loadRoots()
      }

      setActiveId(null)
      return
    }

    const targetParentId = overItem.parentId
    const siblings = visibleItems.filter(
      (item) => item.parentId === targetParentId && item.depth === overItem.depth,
    )
    const siblingIds = siblings.map((item) => item.id)
    const activeIndex = siblingIds.indexOf(activeId)
    const overIndex = siblingIds.indexOf(overId)

    if (activeIndex === -1 || overIndex === -1) {
      setActiveId(null)
      return
    }

    const orderedIds = arrayMove(siblingIds, activeIndex, overIndex)
    const newIndex = orderedIds.indexOf(activeId)

    setTree((prev) => {
      const { node, tree: removedTree } = removeNode(prev, activeId)
      if (!node) return prev

      const updatedNode = { ...node, parent_id: targetParentId, sort_order: newIndex }
      let nextTree = insertNode(removedTree, targetParentId, updatedNode, newIndex)
      nextTree = reorderChildren(nextTree, targetParentId, orderedIds)

      if (node.parent_id && node.parent_id !== targetParentId) {
        const oldParent = findNode(nextTree, node.parent_id)
        if (oldParent && (!oldParent.children || oldParent.children.length === 0)) {
          nextTree = updateNode(nextTree, node.parent_id, (parent) => ({
            ...parent,
            has_children: false,
          }))
        }
      }

      return nextTree
    })

    const beforeId = newIndex < orderedIds.length - 1 ? orderedIds[newIndex + 1] : null
    const afterId = !beforeId && newIndex > 0 ? orderedIds[newIndex - 1] : null

    try {
      await moveGroup(realm, activeId, {
        parent_id: targetParentId,
        before_id: beforeId,
        after_id: afterId,
      })
    } catch (err: any) {
      toast.error(err.message || 'Failed to move group')
      void loadRoots()
    }

    setActiveId(null)
  }

  const handleDragCancel = () => {
    setActiveId(null)
  }

  const handleMoveToRoot = async (groupId: string) => {
    try {
      await moveGroup(realm, groupId, { parent_id: null })
      void loadRoots()
    } catch (err: any) {
      toast.error(err.message || 'Failed to move group')
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="text-muted-foreground absolute left-2 top-2.5 h-4 w-4" />
            <Input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Search groups"
              className="h-9 pl-8"
            />
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto px-2 py-3">
        {loading ? (
          <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-2">
            <Loader2 className="h-5 w-5 animate-spin" />
            <span className="text-xs">Loading groups...</span>
          </div>
        ) : tree.length === 0 ? (
          <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-2 text-sm">
            <span>No groups yet.</span>
            <Button size="sm" variant="outline" onClick={() => onCreateGroup(null)}>
              Create your first group
            </Button>
          </div>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
            onDragCancel={handleDragCancel}
          >
            <SortableContext items={visibleItems.map((item) => item.id)} strategy={verticalListSortingStrategy}>
              <div className="flex flex-col gap-1">
                {visibleItems.map((item) => (
                  <GroupTreeItem
                    key={item.id}
                    item={item}
                    isExpanded={expandedIds.has(item.id)}
                    isSelected={selectedId === item.id}
                    onToggle={toggleExpand}
                    onSelect={onSelect}
                    onCreateChild={(parentId) => onCreateGroup(parentId)}
                    onMoveToRoot={handleMoveToRoot}
                  />
                ))}
              </div>
            </SortableContext>

            <DragOverlay dropAnimation={null}>
              {activeItem ? (
                <div
                  className={cn(
                    'bg-background flex items-center gap-2 rounded-md border px-2 py-1 text-sm shadow-lg',
                  )}
                >
                  <span className="text-muted-foreground">Moving</span>
                  <span className="font-medium">{activeItem.name}</span>
                </div>
              ) : null}
            </DragOverlay>
          </DndContext>
        )}
      </div>
    </div>
  )
}
