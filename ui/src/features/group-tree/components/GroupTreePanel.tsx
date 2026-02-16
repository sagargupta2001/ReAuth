import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

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
import { useQueryClient } from '@tanstack/react-query'
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
  flattenTree,
  insertNode,
  removeChildrenOf,
  removeNode,
  sortTreeByName,
  updateNode,
} from '@/features/group-tree/lib/tree-utils'
import { GroupTreeItem } from '@/features/group-tree/components/GroupTreeItem'
import { useGroupTreeStore } from '@/features/group-tree/model/groupTreeStore'
import { cn } from '@/lib/utils'

interface GroupTreePanelProps {
  selectedId?: string
  onSelect: (groupId: string) => void
  onCreateGroup: (parentId: string | null) => void
  refreshKey?: number
}

const EMPTY_IDS: string[] = []

export function GroupTreePanel({
  selectedId,
  onSelect,
  onCreateGroup,
  refreshKey,
}: GroupTreePanelProps) {
  const realm = useActiveRealm()
  const queryClient = useQueryClient()
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }))

  const [tree, setTree] = useState<GroupTreeNode[]>([])
  const [loading, setLoading] = useState(false)
  const [search, setSearch] = useState('')
  const [activeId, setActiveId] = useState<string | null>(null)
  const loadingIdsRef = useRef<Set<string>>(new Set())
  const hydratedIdsRef = useRef<Set<string>>(new Set())
  const expandedByRealm = useGroupTreeStore((state) => state.expandedByRealm)
  const expandedIdsList = expandedByRealm[realm] ?? EMPTY_IDS
  const setExpanded = useGroupTreeStore((state) => state.setExpanded)
  const toggleExpandedStore = useGroupTreeStore((state) => state.toggleExpanded)
  const resetExpanded = useGroupTreeStore((state) => state.resetExpanded)
  const expandedIds = useMemo(() => new Set(expandedIdsList), [expandedIdsList])

  const addExpanded = useCallback(
    (groupId: string) => {
      if (expandedIdsList.includes(groupId)) return
      const next = new Set(expandedIdsList)
      next.add(groupId)
      setExpanded(realm, Array.from(next))
    },
    [expandedIdsList, realm, setExpanded],
  )

  const loadRoots = useCallback(async () => {
    setLoading(true)
    hydratedIdsRef.current = new Set()
    try {
      const response = await fetchGroupRoots(realm, {
        page: 1,
        per_page: 200,
        sort_by: 'name',
        sort_dir: 'asc',
        q: search.trim() ? search.trim() : undefined,
      })

      setTree(sortTreeByName(response.data))
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
    if (!search.trim()) return
    if (expandedIdsList.length === 0) return
    hydratedIdsRef.current = new Set()
    resetExpanded(realm)
  }, [expandedIdsList.length, realm, resetExpanded, search])

  const loadChildren = useCallback(
    async (groupId: string) => {
      if (loadingIdsRef.current.has(groupId)) return
      loadingIdsRef.current.add(groupId)

      try {
        const response = await fetchGroupChildren(realm, groupId, {
          page: 1,
          per_page: 200,
          sort_by: 'name',
          sort_dir: 'asc',
        })

        setTree((prev) =>
          updateNode(prev, groupId, (node) => ({
            ...node,
            children: sortTreeByName(response.data),
            has_children: response.meta.total > 0,
          })),
        )
        addExpanded(groupId)
      } catch (err: any) {
        toast.error(err.message || 'Failed to load group children')
      } finally {
        loadingIdsRef.current.delete(groupId)
      }
    },
    [addExpanded, realm],
  )

  useEffect(() => {
    if (search.trim()) return
    if (expandedIdsList.length === 0) return

    expandedIdsList.forEach((groupId) => {
      if (hydratedIdsRef.current.has(groupId)) return
      const node = findNode(tree, groupId)
      if (!node) return
      if (node.has_children && !node.children) {
        hydratedIdsRef.current.add(groupId)
        void loadChildren(groupId)
      }
    })
  }, [expandedIdsList, loadChildren, search, tree])

  const toggleExpand = useCallback(
    (groupId: string) => {
      const isExpanded = expandedIds.has(groupId)
      if (isExpanded) {
        toggleExpandedStore(realm, groupId)
        return
      }

      const node = findNode(tree, groupId)
      if (!node) return

      if (!node.children && node.has_children) {
        void loadChildren(groupId)
        return
      }

      toggleExpandedStore(realm, groupId)
    },
    [expandedIds, loadChildren, realm, toggleExpandedStore, tree],
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

  const invalidateChildren = useCallback(
    (parentId: string | null | undefined) => {
      if (!parentId) return
      queryClient.invalidateQueries({
        queryKey: ['group-children', realm, parentId],
        exact: false,
      })
    },
    [queryClient, realm],
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
    const targetId = over.id as string
    if (activeId === targetId) {
      setActiveId(null)
      return
    }

    const activeNode = findNode(tree, activeId)
    const oldParentId = activeNode?.parent_id ?? null

    const overItem = visibleItems.find((item) => item.id === targetId)
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
      const targetParentId = targetId
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

        return sortTreeByName(nextTree)
      })

      try {
        await moveGroup(realm, activeId, {
          parent_id: targetParentId,
        })
        if (!parentNode?.children) {
          void loadChildren(targetParentId)
        } else {
          addExpanded(targetParentId)
        }
        invalidateChildren(oldParentId)
        invalidateChildren(targetParentId)
      } catch (err: any) {
        toast.error(err.message || 'Failed to move group')
        void loadRoots()
      }

      setActiveId(null)
      return
    }
    setActiveId(null)
  }

  const handleDragCancel = () => {
    setActiveId(null)
  }

  const handleMoveToRoot = async (groupId: string) => {
    const node = findNode(tree, groupId)
    const oldParentId = node?.parent_id ?? null
    try {
      await moveGroup(realm, groupId, { parent_id: null })
      invalidateChildren(oldParentId)
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
