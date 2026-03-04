import type { ThemeBlueprint, ThemeDraft, ThemeNode } from '@/entities/theme/model/types'

export type ThemeValidationError = {
  path: string
  message: string
  nodeId?: string
}

const ALLOWED_TYPES = new Set(['Box', 'Text', 'Image', 'Icon', 'Input', 'Component'])

export function validateThemeDraft(
  draft: ThemeDraft,
  options?: { allowLegacyBlocks?: boolean },
): ThemeValidationError[] {
  const errors: ThemeValidationError[] = []
  draft.nodes.forEach((node, index) => {
    errors.push(...validateBlueprint(node.blueprint, `nodes[${index}].blueprint`, options))
  })
  return errors
}

export function validateBlueprint(
  blueprint: ThemeBlueprint,
  path = 'blueprint',
  options?: { allowLegacyBlocks?: boolean },
): ThemeValidationError[] {
  const errors: ThemeValidationError[] = []
  if (Array.isArray(blueprint)) {
    blueprint.forEach((node, index) => {
      errors.push(...validateNode(node, `${path}[${index}]`))
    })
    return errors
  }

  const nodes = blueprint.nodes
  if (Array.isArray(nodes)) {
    nodes.forEach((node, index) => {
      errors.push(...validateNode(node, `${path}.nodes[${index}]`))
    })
    return errors
  }

  if (options?.allowLegacyBlocks && (blueprint as { blocks?: unknown }).blocks) {
    return errors
  }

  errors.push({ path, message: 'Blueprint must define a nodes array.' })
  return errors
}

function validateNode(node: ThemeNode, path: string): ThemeValidationError[] {
  const errors: ThemeValidationError[] = []
  const nodeId = typeof node.id === 'string' ? node.id : undefined
  const push = (message: string) => {
    errors.push({ path, message, nodeId })
  }
  if (!node.type) {
    push('Node type is required.')
    return errors
  }
  if (!ALLOWED_TYPES.has(node.type)) {
    push(`Unsupported node type: ${node.type}`)
  }
  if (node.type === 'Component' && !node.component) {
    push('Component nodes must define a component name.')
  }
  if (node.children) {
    node.children.forEach((child, index) => {
      errors.push(...validateNode(child, `${path}.children[${index}]`))
    })
  }
  if (node.slots) {
    Object.entries(node.slots).forEach(([key, slotNode]) => {
      if (!slotNode) {
        errors.push({
          path: `${path}.slots.${key}`,
          message: 'Slot node is required.',
          nodeId,
        })
        return
      }
      errors.push(...validateNode(slotNode, `${path}.slots.${key}`))
    })
  }
  return errors
}
