import type { FlattenedGroupNode, GroupTreeNode } from '@/features/group-tree/model/types'

export const indentationWidth = 18

export function sortTreeByName(items: GroupTreeNode[]): GroupTreeNode[] {
  return [...items]
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((item) => ({
      ...item,
      children: item.children ? sortTreeByName(item.children) : item.children,
    }))
}

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
