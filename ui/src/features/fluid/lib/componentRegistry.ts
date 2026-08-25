import type { ThemeNode } from '@/entities/theme/model/types'
import { resolvePartStyle } from '@/features/fluid/lib/nodeStyle'
import { withAlpha } from '@/lib/colorUtils'

const DEFAULT_LABEL_SIZE = '12px'
const DEFAULT_LABEL_WEIGHT = '600'
const DEFAULT_ERROR_SIZE = '12px'
const DEFAULT_ERROR_WEIGHT = '400'
const DEFAULT_ERROR_COLOR = '#ef4444'
const DEFAULT_FIELD_BORDER_WIDTH = 1
const DEFAULT_FIELD_RADIUS = 8
const DEFAULT_FIELD_PADDING = 10
const DEFAULT_FIELD_GAP = 8
const DEFAULT_STACK_GAP = 0

/** Alpha applied to the theme's text colour to derive a field border. */
const BORDER_ALPHA = 0.22
/** Alpha applied to the theme's text colour to derive secondary label text. */
const LABEL_ALPHA = 0.7

/**
 * Theme colours a component expansion needs for the parts the blueprint does not
 * specify.
 *
 * Without this the expansion has to hard-code values, which is how inputs ended
 * up rendering as solid white boxes on dark themes.
 */
export interface ComponentThemeContext {
  /** The theme's text colour; borders and label text are derived from it. */
  text?: string
  /** Base corner radius, in pixels. */
  radius?: number
}

function fieldBorderColor(theme?: ComponentThemeContext) {
  return theme?.text ? withAlpha(theme.text, BORDER_ALPHA) : `rgba(127, 127, 127, ${BORDER_ALPHA})`
}

function labelColor(theme?: ComponentThemeContext) {
  return theme?.text ? withAlpha(theme.text, LABEL_ALPHA) : 'var(--muted-foreground)'
}

type ComponentDefinition = {
  id: string
  expand: (node: ThemeNode, theme?: ComponentThemeContext) => ThemeNode | null
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
  expand: (node, theme) => {
    const props = node.props ?? {}
    const baseId = node.id ?? 'input'
    const labelText = String(props.label ?? '')
    // Part styling comes from `style.parts.label`, with the legacy `label_*`
    // props folded in for blueprints that predate style groups.
    const labelStyle = resolvePartStyle(node, 'label')
    const labelNode: ThemeNode = {
      id: `${baseId}-label`,
      type: 'Text',
      size: { width: 'fill', height: 'hug' },
      props: {
        text: labelText,
        align: props.align,
        visible: labelText.trim().length > 0,
      },
      style: {
        typography: {
          size: String(labelStyle.typography.size || DEFAULT_LABEL_SIZE),
          weight: String(labelStyle.typography.weight || DEFAULT_LABEL_WEIGHT),
          color: String(labelStyle.typography.color || labelColor(theme)),
        },
        spacing: { margin_bottom: parseNumber(labelStyle.spacing.margin_bottom, 4) },
      },
    }

    const fieldStyle = resolvePartStyle(node, 'field')
    // Spacing on the field part means the padding *inside* the box, which is a
    // container concern the expansion owns rather than a wrapper style.
    const fieldPadding = parseNumber(fieldStyle.spacing.padding, DEFAULT_FIELD_PADDING)
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
      style: {
        stroke: {
          color: String(fieldStyle.stroke.color || fieldBorderColor(theme)),
          width: parseNumber(fieldStyle.stroke.width, DEFAULT_FIELD_BORDER_WIDTH),
        },
        corners: {
          radius: parseNumber(fieldStyle.corners.radius, theme?.radius ?? DEFAULT_FIELD_RADIUS),
        },
        // Transparent by default so the field sits on the theme's own surface.
        fill: { color: String(fieldStyle.fill.color || 'transparent') },
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

/**
 * Component names this registry expands into primitives.
 *
 * A component named here is rendered by the expansion and needs no `case` in
 * either renderer; anything else needs one in *both*, which is how
 * `ProviderButtons` once shipped working at runtime and broken in the builder.
 */
export const EXPANDED_COMPONENTS: readonly string[] = Object.keys(COMPONENTS)

export function expandComponentNode(
  node: ThemeNode,
  theme?: ComponentThemeContext,
): ThemeNode | null {
  if (node.type !== 'Component' || !node.component) return null
  const definition = COMPONENTS[node.component]
  if (!definition) return null
  return definition.expand(node, theme)
}
