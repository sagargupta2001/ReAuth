import { arrayMove } from '@dnd-kit/sortable'

import type { FlattenedGroupNode, GroupTreeNode } from '@/features/group-tree/model/types'

export const indentationWidth = 18

export function flattenTree(
  items: GroupTreeNode[],
  parentId: string | null = null,
  depth = 0,
): FlattenedGroupNode[] {
  return items.reduce<FlattenedGroupNode[]>((acc, item, index) => {
    const flattened: FlattenedGroupNode = {
      ...item,
      parentId,
      depth,
      index,
    }

    acc.push(flattened)

    if (item.children && item.children.length > 0) {
      acc.push(...flattenTree(item.children, item.id, depth + 1))
    }

    return acc
  }, [])
}

export function removeChildrenOf(
  items: FlattenedGroupNode[],
  collapsedIds: Set<string>,
): FlattenedGroupNode[] {
  const excludedIds = new Set<string>()

  return items.filter((item) => {
    if (item.parentId && excludedIds.has(item.parentId)) {
      excludedIds.add(item.id)
      return false
    }

    if (item.parentId && collapsedIds.has(item.parentId)) {
      excludedIds.add(item.id)
      return false
    }

    if (collapsedIds.has(item.id)) {
      excludedIds.add(item.id)
    }

    return true
  })
}

function getMaxDepth(items: FlattenedGroupNode[], overIndex: number) {
  if (overIndex === 0) return 0
  return items[overIndex - 1].depth + 1
}

function getMinDepth(items: FlattenedGroupNode[], overIndex: number) {
  if (overIndex === items.length - 1) return 0
  return items[overIndex + 1].depth
}

function getParentIdForDepth(
  items: FlattenedGroupNode[],
  overIndex: number,
  depth: number,
): string | null {
  for (let i = overIndex - 1; i >= 0; i -= 1) {
    const item = items[i]
    if (item.depth === depth - 1) return item.id
  }

  return null
}

export function getProjection(
  items: FlattenedGroupNode[],
  activeId: string,
  overId: string,
  offsetLeft: number,
): { depth: number; parentId: string | null } | null {
  const overIndex = items.findIndex((item) => item.id === overId)
  const activeIndex = items.findIndex((item) => item.id === activeId)

  if (overIndex === -1 || activeIndex === -1) return null

  const activeItem = items[activeIndex]
  const newItems = arrayMove(items, activeIndex, overIndex)

  const projectedDepth = activeItem.depth + Math.round(offsetLeft / indentationWidth)
  const maxDepth = getMaxDepth(newItems, overIndex)
  const minDepth = getMinDepth(newItems, overIndex)
  const depth = Math.max(minDepth, Math.min(projectedDepth, maxDepth))

  const parentId = depth === 0 ? null : getParentIdForDepth(newItems, overIndex, depth)

  return { depth, parentId }
}

export function findNode(items: GroupTreeNode[], id: string): GroupTreeNode | null {
  for (const item of items) {
    if (item.id === id) return item
    if (item.children) {
      const found = findNode(item.children, id)
      if (found) return found
    }
  }
  return null
}

export function findPath(items: GroupTreeNode[], id: string): GroupTreeNode[] | null {
  for (const item of items) {
    if (item.id === id) return [item]
    if (item.children) {
      const childPath = findPath(item.children, id)
      if (childPath) return [item, ...childPath]
    }
  }
  return null
}

export function updateNode(
  items: GroupTreeNode[],
  id: string,
  updater: (node: GroupTreeNode) => GroupTreeNode,
): GroupTreeNode[] {
  return items.map((item) => {
    if (item.id === id) return updater(item)
    if (item.children) {
      return { ...item, children: updateNode(item.children, id, updater) }
    }
    return item
  })
}

export function removeNode(
  items: GroupTreeNode[],
  id: string,
): { node: GroupTreeNode | null; tree: GroupTreeNode[] } {
  let removed: GroupTreeNode | null = null

  const next = items
    .map((item) => {
      if (item.id === id) {
        removed = item
        return null
      }
      if (item.children) {
        const result = removeNode(item.children, id)
        if (result.node) {
          removed = result.node
          return { ...item, children: result.tree }
        }
      }
      return item
    })
    .filter(Boolean) as GroupTreeNode[]

  return { node: removed, tree: next }
}

export function insertNode(
  items: GroupTreeNode[],
  parentId: string | null,
  node: GroupTreeNode,
  index: number,
): GroupTreeNode[] {
  if (!parentId) {
    const next = [...items]
    next.splice(index, 0, node)
    return next
  }

  return updateNode(items, parentId, (parent) => {
    const children = parent.children ? [...parent.children] : []
    children.splice(index, 0, node)
    return { ...parent, children, has_children: true }
  })
}

export function reorderChildren(
  items: GroupTreeNode[],
  parentId: string | null,
  orderedIds: string[],
): GroupTreeNode[] {
  if (!parentId) {
    const byId = new Map(items.map((item) => [item.id, item]))
    return orderedIds.map((id) => byId.get(id)).filter(Boolean) as GroupTreeNode[]
  }

  return updateNode(items, parentId, (parent) => {
    if (!parent.children) return parent
    const byId = new Map(parent.children.map((child) => [child.id, child]))
    const ordered = orderedIds.map((id) => byId.get(id)).filter(Boolean) as GroupTreeNode[]
    return { ...parent, children: ordered }
  })
}
