import type {
  ThemeBlueprint,
  ThemeNode,
  ThemeNodeLayout,
  ThemeNodeSize,
} from '@/entities/theme/model/types'

export type ThemeNodeDefinition = Omit<ThemeNode, 'id' | 'children' | 'slots'> & {
  children?: ThemeNodeDefinition[]
  slots?: Record<string, ThemeNodeDefinition>
}

function generateNodeId(prefix = 'node') {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return `${prefix}-${crypto.randomUUID()}`
  }
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`
}

function defaultSizeForNode(node: ThemeNodeDefinition): ThemeNodeSize | undefined {
  if (node.size) return node.size
  switch (node.type) {
    case 'Icon':
      return { width: 'hug', height: 'hug' }
    case 'Input':
      return { width: 'fill', height: 'hug' }
    case 'Text':
      return { width: 'fill', height: 'hug' }
    case 'Image':
      return { width: 'fill', height: 'hug' }
    case 'Box':
      return { width: 'fill', height: 'hug' }
    case 'Component':
      return { width: 'fill', height: 'hug' }
    default:
      return undefined
  }
}

export function createNodeFromDefinition(node: ThemeNodeDefinition): ThemeNode {
  const normalized: ThemeNode = {
    ...(node as Omit<ThemeNode, 'id'>),
    id: generateNodeId(node.component ?? node.type ?? 'node'),
    props: node.props ?? {},
    size: defaultSizeForNode(node),
    children: (node.children ?? []) as ThemeNode[],
    slots: (node.slots ?? {}) as Record<string, ThemeNode>,
  }
  return ensureNodeIds([normalized])[0]
}

function ensureNodeId(node: ThemeNode, fallbackPrefix: string) {
  if (!node.id) {
    node.id = generateNodeId(fallbackPrefix)
  }
}

function ensureNodeIds(nodes: ThemeNode[], prefix = 'node'): ThemeNode[] {
  return nodes.map((node, index) => {
    const normalized: ThemeNode = {
      ...node,
      props: node.props ?? {},
      children: node.children ?? [],
      slots: node.slots ?? {},
    }
    ensureNodeId(normalized, `${prefix}-${index}`)
    if (normalized.children && normalized.children.length > 0) {
      normalized.children = ensureNodeIds(normalized.children, normalized.id)
    }
    if (normalized.slots && Object.keys(normalized.slots).length > 0) {
      const nextSlots: Record<string, ThemeNode> = {}
      Object.entries(normalized.slots).forEach(([key, slotNode]) => {
        const normalizedSlot = ensureNodeIds([slotNode], `${normalized.id}-${key}`)[0]
        nextSlots[key] = normalizedSlot
      })
      normalized.slots = nextSlots
    }
    return normalized
  })
}

export function extractNodesFromBlueprint(blueprint?: ThemeBlueprint) {
  if (!blueprint) {
    return { nodes: [] as ThemeNode[], layout: undefined as string | undefined }
  }

  if (Array.isArray(blueprint)) {
    const nodes = blueprint as ThemeNode[]
    return { nodes: ensureNodeIds(nodes), layout: undefined }
  }

  const layout = typeof blueprint.layout === 'string' ? blueprint.layout : undefined
  return {
    nodes: ensureNodeIds(blueprint.nodes ?? []),
    layout,
  }
}

export function updateBlueprintWithNodes(
  blueprint: ThemeBlueprint | undefined,
  nodes: ThemeNode[],
  layoutFallback = 'default',
): ThemeBlueprint {
  if (!blueprint || Array.isArray(blueprint)) {
    return { layout: layoutFallback, nodes }
  }
  return {
    ...blueprint,
    layout: blueprint.layout ?? layoutFallback,
    nodes,
  }
}

export function findNodeById(nodes: ThemeNode[], id: string): ThemeNode | null {
  for (const node of nodes) {
    if (node.id === id) return node
    const childMatch = findNodeById(node.children ?? [], id)
    if (childMatch) return childMatch
    const slots = node.slots ?? {}
    for (const slotNode of Object.values(slots)) {
      const slotMatch = findNodeById([slotNode], id)
      if (slotMatch) return slotMatch
    }
  }
  return null
}

export function updateNodeById(
  nodes: ThemeNode[],
  id: string,
  updater: (node: ThemeNode) => ThemeNode,
): ThemeNode[] {
  return nodes.map((node) => {
    if (node.id === id) {
      return updater(node)
    }
    const updatedChildren = updateNodeById(node.children ?? [], id, updater)
    const updatedSlots: Record<string, ThemeNode> = { ...(node.slots ?? {}) }
    let slotsChanged = false
    Object.entries(updatedSlots).forEach(([key, slotNode]) => {
      const updatedSlot = updateNodeById([slotNode], id, updater)[0]
      if (updatedSlot !== slotNode) {
        updatedSlots[key] = updatedSlot
        slotsChanged = true
      }
    })
    if (
      updatedChildren !== (node.children ?? []) ||
      slotsChanged
    ) {
      return {
        ...node,
        children: updatedChildren,
        slots: updatedSlots,
      }
    }
    return node
  })
}

export function removeNodeById(nodes: ThemeNode[], id: string): ThemeNode[] {
  const filtered = nodes
    .filter((node) => node.id !== id)
    .map((node) => {
      const updatedChildren = removeNodeById(node.children ?? [], id)
      const updatedSlots: Record<string, ThemeNode> = {}
      Object.entries(node.slots ?? {}).forEach(([key, slotNode]) => {
        if (slotNode.id === id) {
          return
        }
        const updatedSlot = removeNodeById([slotNode], id)[0]
        if (updatedSlot) {
          updatedSlots[key] = updatedSlot
        }
      })
      return {
        ...node,
        children: updatedChildren,
        slots: updatedSlots,
      }
    })
  return filtered
}

export function mergeNodeLayout(
  current?: ThemeNodeLayout,
  partial?: Partial<ThemeNodeLayout>,
): ThemeNodeLayout | undefined {
  if (!current && !partial) return current
  return { ...(current ?? {}), ...(partial ?? {}) }
}

export function mergeNodeSize(
  current?: ThemeNodeSize,
  partial?: Partial<ThemeNodeSize>,
): ThemeNodeSize | undefined {
  if (!current && !partial) return current
  return { ...(current ?? {}), ...(partial ?? {}) }
}

/* ---------------------------------------------------------------------------
 * Structural editing
 *
 * Everything below addresses nodes by **id** and walks only the *authored*
 * tree (`children`). Slots are a component's contract, not free composition:
 * they are visible and selectable in the sections tree but cannot be moved,
 * reordered, or receive drops, so no structural helper descends into them.
 * ------------------------------------------------------------------------ */

/** Where a dragged node should land relative to a target row. */
export type NodeDropIntent = 'before' | 'after' | 'inside'

/** An insertion address. `parentId: null` addresses the page's root list. */
export interface NodeLocation {
  parentId: string | null
  index: number
}

export type DropRejectionReason = 'unknown-node' | 'cycle' | 'depth-limit' | 'no-op'

export type DropResolution =
  | { ok: true; location: NodeLocation; intent: NodeDropIntent }
  | { ok: false; reason: DropRejectionReason; message: string }

export interface DropRequest {
  dragId: string
  /** Row the pointer is over, or `null` to append to the page root. */
  targetId: string | null
  intent: NodeDropIntent
}

export interface DropOptions {
  /** Whether a node's block declares that it can contain other blocks. */
  acceptsChildren: (node: ThemeNode) => boolean
  /** Maximum number of levels in the authored tree; root nodes are level 1. */
  maxDepth: number
}

/**
 * Root-to-node chain, or `null` when the id is unknown or owned by a slot.
 *
 * A slot node returning `null` is what makes every structural operation
 * reject it without each call site re-checking.
 */
export function findNodePath(nodes: ThemeNode[], id: string): ThemeNode[] | null {
  for (const node of nodes) {
    if (node.id === id) return [node]
    const childPath = findNodePath(node.children ?? [], id)
    if (childPath) return [node, ...childPath]
  }
  return null
}

/** Depth in the authored tree, root nodes being `0`. */
export function depthOfNode(nodes: ThemeNode[], id: string): number | null {
  const path = findNodePath(nodes, id)
  return path ? path.length - 1 : null
}

/** Levels of authored children below `node`; a leaf is `0`. */
export function subtreeHeight(node: ThemeNode): number {
  const children = node.children ?? []
  if (children.length === 0) return 0
  return 1 + Math.max(...children.map(subtreeHeight))
}

/** True when `descendantId` sits anywhere inside `ancestorId`'s subtree. */
export function isDescendantOf(
  nodes: ThemeNode[],
  descendantId: string,
  ancestorId: string,
): boolean {
  const path = findNodePath(nodes, descendantId)
  if (!path) return false
  return path.slice(0, -1).some((ancestor) => ancestor.id === ancestorId)
}

/** The node's current address, or `null` when it is not in the authored tree. */
export function locationOfNode(nodes: ThemeNode[], id: string): NodeLocation | null {
  const path = findNodePath(nodes, id)
  if (!path) return null
  const parent = path.length > 1 ? path[path.length - 2] : null
  const siblings = parent ? parent.children ?? [] : nodes
  return {
    parentId: parent?.id ?? null,
    index: siblings.findIndex((sibling) => sibling.id === id),
  }
}

function spliceAt(list: ThemeNode[], node: ThemeNode, index: number): ThemeNode[] {
  const next = [...list]
  next.splice(Math.max(0, Math.min(index, next.length)), 0, node)
  return next
}

/**
 * Inserts `node` at `location`. Returns the original array when the parent
 * cannot be found, so a stale location is a no-op rather than a lost node.
 */
export function insertNodeAt(
  nodes: ThemeNode[],
  node: ThemeNode,
  location: NodeLocation,
): ThemeNode[] {
  if (location.parentId === null) {
    return spliceAt(nodes, node, location.index)
  }

  let inserted = false
  const visit = (list: ThemeNode[]): ThemeNode[] =>
    list.map((current) => {
      if (inserted) return current
      if (current.id === location.parentId) {
        inserted = true
        return {
          ...current,
          children: spliceAt(current.children ?? [], node, location.index),
        }
      }
      const children = current.children ?? []
      if (children.length === 0) return current
      return { ...current, children: visit(children) }
    })

  const next = visit(nodes)
  return inserted ? next : nodes
}

/**
 * Locations are expressed against the tree *before* the node is detached, so
 * a move within the same parent past its own position shifts left by one.
 */
function adjustForRemoval(current: NodeLocation, target: NodeLocation): NodeLocation {
  if (current.parentId !== target.parentId) return target
  return target.index > current.index ? { ...target, index: target.index - 1 } : target
}

function isSameLocation(a: NodeLocation, b: NodeLocation): boolean {
  return a.parentId === b.parentId && a.index === b.index
}

/**
 * Moves an existing node to `location`, expressed against the current tree.
 *
 * Returns the original array unchanged for a stale id, a no-op move, or a move
 * that would put the node inside its own subtree.
 */
export function moveNode(
  nodes: ThemeNode[],
  nodeId: string,
  location: NodeLocation,
): ThemeNode[] {
  const path = findNodePath(nodes, nodeId)
  const current = locationOfNode(nodes, nodeId)
  if (!path || !current) return nodes

  const { parentId } = location
  if (parentId === nodeId || (parentId && isDescendantOf(nodes, parentId, nodeId))) {
    return nodes
  }

  const adjusted = adjustForRemoval(current, location)
  if (isSameLocation(current, adjusted)) return nodes

  const node = path[path.length - 1]
  return insertNodeAt(removeNodeById(nodes, nodeId), node, adjusted)
}

/**
 * Turns "this node was dropped here, this way" into an insertion address, or
 * into the reason the drop is not allowed.
 *
 * Rejections are values rather than thrown errors because every one of them is
 * something the UI has to show: a not-allowed cursor while dragging, or a
 * message on drop.
 */
export function resolveDrop(
  nodes: ThemeNode[],
  request: DropRequest,
  options: DropOptions,
): DropResolution {
  const dragPath = findNodePath(nodes, request.dragId)
  const current = locationOfNode(nodes, request.dragId)
  if (!dragPath || !current) {
    return { ok: false, reason: 'unknown-node', message: 'That block is no longer on the page.' }
  }
  const dragged = dragPath[dragPath.length - 1]

  let intent: NodeDropIntent = request.intent
  let location: NodeLocation

  if (request.targetId === null) {
    intent = 'inside'
    location = { parentId: null, index: nodes.length }
  } else {
    const targetPath = findNodePath(nodes, request.targetId)
    if (!targetPath) {
      return {
        ok: false,
        reason: 'unknown-node',
        message: 'That block is no longer on the page.',
      }
    }
    const target = targetPath[targetPath.length - 1]
    // Rule 5 of the spec: a drop on a non-container is a sibling drop, never a
    // failed attempt to nest.
    if (intent === 'inside' && !options.acceptsChildren(target)) {
      intent = 'after'
    }

    if (intent === 'inside') {
      location = { parentId: target.id, index: (target.children ?? []).length }
    } else {
      const parent = targetPath.length > 1 ? targetPath[targetPath.length - 2] : null
      const siblings = parent ? parent.children ?? [] : nodes
      const targetIndex = siblings.findIndex((sibling) => sibling.id === target.id)
      location = {
        parentId: parent?.id ?? null,
        index: targetIndex + (intent === 'after' ? 1 : 0),
      }
    }
  }

  const { parentId } = location
  if (parentId === request.dragId || (parentId && isDescendantOf(nodes, parentId, request.dragId))) {
    return { ok: false, reason: 'cycle', message: 'A block cannot be moved inside itself.' }
  }

  const parentDepth = parentId === null ? -1 : depthOfNode(nodes, parentId) ?? -1
  const deepestLevel = parentDepth + 1 + subtreeHeight(dragged)
  if (deepestLevel > options.maxDepth - 1) {
    return {
      ok: false,
      reason: 'depth-limit',
      message: `Blocks can be nested up to ${options.maxDepth} levels deep.`,
    }
  }

  if (isSameLocation(current, adjustForRemoval(current, location))) {
    return { ok: false, reason: 'no-op', message: 'The block is already there.' }
  }

  return { ok: true, location, intent }
}
