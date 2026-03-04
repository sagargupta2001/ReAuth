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
