import type { ThemeNode } from '@/entities/theme/model/types'

const DEFAULT_LABEL_SIZE = '12px'
const DEFAULT_LABEL_WEIGHT = '600'
const DEFAULT_LABEL_COLOR = 'var(--muted-foreground)'
const DEFAULT_ERROR_SIZE = '12px'
const DEFAULT_ERROR_WEIGHT = '400'
const DEFAULT_ERROR_COLOR = '#ef4444'
const DEFAULT_FIELD_BORDER_COLOR = '#e2e8f0'
const DEFAULT_FIELD_BORDER_WIDTH = 1
const DEFAULT_FIELD_RADIUS = 8
const DEFAULT_FIELD_BACKGROUND = '#ffffff'
const DEFAULT_FIELD_PADDING = 8
const DEFAULT_FIELD_GAP = 8
const DEFAULT_STACK_GAP = 0

type ComponentDefinition = {
  id: string
  expand: (node: ThemeNode) => ThemeNode | null
}

const parseNumber = (value: unknown, fallback: number) => {
  if (typeof value === 'number' && !Number.isNaN(value)) return value
  const parsed = Number.parseFloat(String(value ?? ''))
  return Number.isNaN(parsed) ? fallback : parsed
}

const normalizeSlotNode = (slot: ThemeNode, fallbackId: string) => ({
  ...slot,
  id: slot.id ?? fallbackId,
  props: slot.props ?? {},
  children: slot.children ?? [],
  slots: slot.slots ?? {},
})

const inputComponent: ComponentDefinition = {
  id: 'Input',
  expand: (node) => {
    const props = node.props ?? {}
    const baseId = node.id ?? 'input'
    const labelText = String(props.label ?? '')
    const labelSpacing = parseNumber(props.label_spacing, 4)
    const labelNode: ThemeNode = {
      id: `${baseId}-label`,
      type: 'Text',
      size: { width: 'fill', height: 'hug' },
      props: {
        text: labelText,
        font_size: String(props.label_size || DEFAULT_LABEL_SIZE),
        font_weight: String(props.label_weight || DEFAULT_LABEL_WEIGHT),
        color: String(props.label_color || DEFAULT_LABEL_COLOR),
        margin_bottom: labelSpacing,
        align: props.align,
        visible: labelText.trim().length > 0,
      },
    }

    const fieldPadding = parseNumber(props.field_padding, DEFAULT_FIELD_PADDING)
    const fieldContainer: ThemeNode = {
      id: `${baseId}-field`,
      type: 'Box',
      size: { width: 'fill', height: 'hug' },
      layout: {
        direction: 'row',
        gap: DEFAULT_FIELD_GAP,
        align: 'center',
        padding: [fieldPadding, fieldPadding, fieldPadding, fieldPadding],
      },
      props: {
        border_color: String(props.field_border_color || DEFAULT_FIELD_BORDER_COLOR),
        border_width: parseNumber(props.field_border_width, DEFAULT_FIELD_BORDER_WIDTH),
        radius: parseNumber(props.field_radius, DEFAULT_FIELD_RADIUS),
        background: String(props.field_background || DEFAULT_FIELD_BACKGROUND),
      },
      children: [],
    }

    const prefixSlot = node.slots?.prefix
    if (prefixSlot) {
      const normalized = normalizeSlotNode(prefixSlot, `${baseId}-prefix`)
      normalized.size = normalized.size ?? { width: 'hug', height: 'hug' }
      normalized.props = {
        ...normalized.props,
        visible: normalized.props?.visible ?? true,
      }
      fieldContainer.children?.push(normalized)
    }

    const inputNode: ThemeNode = {
      id: `${baseId}-input`,
      type: 'Input',
      size: { width: 'fill', height: 'hug' },
      props: {
        name: props.name,
        input_type: props.input_type,
        placeholder: props.placeholder,
        size: props.size,
      },
    }
    fieldContainer.children?.push(inputNode)

    const errorSlot = node.slots?.error
    const errorNode = errorSlot
      ? (() => {
          const normalized = normalizeSlotNode(errorSlot, `${baseId}-error`)
          normalized.size = normalized.size ?? { width: 'fill', height: 'hug' }
          normalized.props = {
            ...normalized.props,
            text: normalized.props?.text ?? 'Invalid value',
            color: normalized.props?.color ?? DEFAULT_ERROR_COLOR,
            font_size: normalized.props?.font_size ?? DEFAULT_ERROR_SIZE,
            font_weight: normalized.props?.font_weight ?? DEFAULT_ERROR_WEIGHT,
            margin_top: normalized.props?.margin_top ?? 4,
            align: props.align,
            visible: normalized.props?.visible ?? false,
          }
          return normalized
        })()
      : null

    const container: ThemeNode = {
      id: `${baseId}-container`,
      type: 'Box',
      size: { width: 'fill', height: 'hug' },
      layout: { direction: 'column', gap: DEFAULT_STACK_GAP, align: 'stretch', padding: [0, 0, 0, 0] },
      children: [labelNode, fieldContainer, ...(errorNode ? [errorNode] : [])],
    }

    return container
  },
}

const COMPONENTS: Record<string, ComponentDefinition> = {
  Input: inputComponent,
}

export function expandComponentNode(node: ThemeNode): ThemeNode | null {
  if (node.type !== 'Component' || !node.component) return null
  const definition = COMPONENTS[node.component]
  if (!definition) return null
  return definition.expand(node)
}
