import type { ThemeNode } from '@/entities/theme/model/types'

export type FluidSignal = {
  type?: string
  node_id?: string
  payload_map?: Record<string, unknown>
}

export type FluidAction = {
  trigger?: string
  signal?: FluidSignal
}

export const normalizeTrigger = (value?: string) => {
  const trimmed = value?.trim().toLowerCase() ?? ''
  if (!trimmed) return ''
  if (trimmed === 'onclick' || trimmed === 'click') return 'on_click'
  if (trimmed === 'onsubmit' || trimmed === 'submit') return 'on_submit'
  if (trimmed === 'onchange' || trimmed === 'change') return 'on_change'
  if (trimmed === 'onload' || trimmed === 'load') return 'on_load'
  return trimmed.replace('-', '_')
}

export const nodeActions = (node: ThemeNode): FluidAction[] => {
  const props = node.props ?? {}
  const rawActions =
    (props as Record<string, unknown>).actions ??
    (node as unknown as Record<string, unknown>).actions
  if (!Array.isArray(rawActions)) return []
  return rawActions.filter((action) => action && typeof action === 'object') as FluidAction[]
}

export const findActionInNode = (node: ThemeNode, trigger: string): FluidAction | null => {
  const normalized = normalizeTrigger(trigger)
  const actions = nodeActions(node)
  const match = actions.find((action) => normalizeTrigger(action.trigger) === normalized)
  if (match) return match
  if (node.children) {
    for (const child of node.children) {
      const found = findActionInNode(child, normalized)
      if (found) return found
    }
  }
  if (node.slots) {
    for (const slot of Object.values(node.slots)) {
      const found = findActionInNode(slot, normalized)
      if (found) return found
    }
  }
  return null
}

export const findActionInTree = (nodes: ThemeNode[], trigger: string): FluidAction | null => {
  for (const node of nodes) {
    const found = findActionInNode(node, trigger)
    if (found) return found
  }
  return null
}
