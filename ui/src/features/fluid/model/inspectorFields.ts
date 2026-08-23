import type { ThemeNode } from '@/entities/theme/model/types'

/**
 * Where a field writes. A node keeps its state in three separate places and the
 * distinction is load-bearing — `props.align` is text alignment while
 * `layout.align` is flex cross-axis alignment, and the old inspector labelled
 * both of them "Alignment".
 */
export const FieldTarget = {
  /** `node.props` — per-node values the renderer reads. */
  Props: 'props',
  /** `node.layout` — flex container settings, only meaningful on a Box. */
  Layout: 'layout',
  /** `node.size` — width/height modes and explicit values. */
  Size: 'size',
} as const

export type FieldTarget = (typeof FieldTarget)[keyof typeof FieldTarget]

/**
 * Control types the inspector can render.
 *
 * To add one: add a member here, add its descriptor to `InspectorField`, then
 * render it in `InspectorField.tsx`. The discriminated union's exhaustiveness
 * check points at the one place still missing.
 */
export const InspectorFieldKind = {
  Text: 'text',
  Number: 'number',
  Textarea: 'textarea',
  Select: 'select',
  /** Read-only display, e.g. the node's type. */
  Readonly: 'readonly',
  /** Icon name plus a searchable picker. */
  Icon: 'icon',
  /** Choose an uploaded theme asset. */
  Asset: 'asset',
  /** The four-number `layout.padding` tuple. */
  PaddingBox: 'padding-box',
  /** A width or height mode with its conditional explicit value. */
  Dimension: 'dimension',
  /** Bespoke sub-panels that do not reduce to a single value. */
  Custom: 'custom',
} as const

export type InspectorFieldKind =
  (typeof InspectorFieldKind)[keyof typeof InspectorFieldKind]

export interface SelectOption {
  value: string
  label: string
}

interface FieldBase {
  /** Stable id, also the control's DOM id. */
  id: string
  label: string
  /** Shown under the control; use it to disambiguate similar-sounding fields. */
  hint?: string
}

interface TargetedField extends FieldBase {
  target: FieldTarget
  /** Key within the target record. */
  key: string
}

export interface TextInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Text
  placeholder?: string
}

export interface NumberInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Number
  placeholder?: string
  min?: number
  max?: number
  step?: number
}

export interface TextareaInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Textarea
  placeholder?: string
}

export interface SelectInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Select
  options: readonly SelectOption[]
  /** Value shown when the node has none. */
  fallback: string
}

export interface ReadonlyInspectorField extends FieldBase {
  kind: typeof InspectorFieldKind.Readonly
  /** Derives the displayed text from the selected node. */
  value: (node: ThemeNode) => string
}

export interface IconInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Icon
}

export interface AssetInspectorField extends TargetedField {
  kind: typeof InspectorFieldKind.Asset
}

export interface PaddingBoxInspectorField extends FieldBase {
  kind: typeof InspectorFieldKind.PaddingBox
  /** Always `layout.padding`; the target is fixed by the control's shape. */
  key: 'padding'
}

export interface DimensionInspectorField extends FieldBase {
  kind: typeof InspectorFieldKind.Dimension
  axis: 'width' | 'height'
  options: readonly SelectOption[]
  fallback: string
  /** Label for the explicit value shown when the mode is `fixed`. */
  valueLabel: string
}

/** Named bespoke panels, for editors that are not a single value. */
export const CustomInspectorPanel = {
  /** Prefix icon and error text slots on an Input component. */
  InputSlots: 'input-slots',
} as const

export type CustomInspectorPanel =
  (typeof CustomInspectorPanel)[keyof typeof CustomInspectorPanel]

export interface CustomInspectorField {
  kind: typeof InspectorFieldKind.Custom
  id: string
  panel: CustomInspectorPanel
}

export type InspectorField =
  | TextInspectorField
  | NumberInspectorField
  | TextareaInspectorField
  | SelectInspectorField
  | ReadonlyInspectorField
  | IconInspectorField
  | AssetInspectorField
  | PaddingBoxInspectorField
  | DimensionInspectorField
  | CustomInspectorField

/**
 * Which nodes a section applies to. A section with neither key applies to every
 * node.
 *
 * This is what stops the Typography card appearing for a Box: applicability is
 * declared once, rather than each section guarding itself with an inline
 * `selectedType === ...` check that is easy to forget.
 */
export interface NodeMatcher {
  types?: readonly ThemeNode['type'][]
  /** Component names, for `type: 'Component'` nodes. */
  components?: readonly string[]
}

export interface InspectorSection {
  id: string
  title: string
  description?: string
  appliesTo?: NodeMatcher
  fields: readonly InspectorField[]
}

/** True when a section should be shown for the selected node. */
export function matchesNode(matcher: NodeMatcher | undefined, node: ThemeNode): boolean {
  if (!matcher) return true

  if (matcher.components?.length) {
    const component = node.component ?? ''
    if (node.type === 'Component' && matcher.components.includes(component)) return true
  }
  if (matcher.types?.length && matcher.types.includes(node.type)) return true

  return false
}
