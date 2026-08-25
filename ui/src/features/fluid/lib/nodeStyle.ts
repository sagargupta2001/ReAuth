import type {
  CornerStyle,
  FillStyle,
  NodeStyle,
  SpacingStyle,
  StrokeStyle,
  ThemeNode,
  TypographyStyle,
} from '@/entities/theme/model/types'

/**
 * The bridge between the flat `props` styling bag and grouped `node.style`.
 *
 * Styling used to be declared per block *and* per slot: `Box` had `background`,
 * `border_color`, `border_width`, `radius`, while an `Input` separately had
 * `field_background`, `field_border_color`, `field_border_width`,
 * `field_radius`, `field_padding`, `label_size`, `label_weight`, `label_color`,
 * and `label_spacing`. Nine props on one component, all re-declaring ideas that
 * already existed. A third composed component would have added nine more.
 *
 * Groups fix that: a styling capability is added once and every node has it.
 *
 * Blueprints are persisted JSON, so the legacy keys stay readable forever.
 * `resolveNodeStyle` is the read path and prefers the group; `normalizeNode` is
 * the write path and moves legacy keys into groups so a page converges on the
 * new shape the first time it is saved.
 */

export const STYLE_GROUPS = ['fill', 'stroke', 'corners', 'spacing', 'typography'] as const

export type StyleGroup = (typeof STYLE_GROUPS)[number]

/** A fully-populated view, so callers never null-check a group. */
export interface ResolvedNodeStyle {
  fill: FillStyle
  stroke: StrokeStyle
  corners: CornerStyle
  spacing: SpacingStyle
  typography: TypographyStyle
}

type StylePath = readonly [StyleGroup, string]

/** Legacy `props` key -> the group and key it became. */
export const LEGACY_STYLE_PROPS: Readonly<Record<string, StylePath>> = {
  background: ['fill', 'color'],
  border_color: ['stroke', 'color'],
  border_width: ['stroke', 'width'],
  radius: ['corners', 'radius'],
  padding: ['spacing', 'padding'],
  margin_top: ['spacing', 'margin_top'],
  margin_bottom: ['spacing', 'margin_bottom'],
  font_size: ['typography', 'size'],
  font_weight: ['typography', 'weight'],
  color: ['typography', 'color'],
  align: ['typography', 'align'],
}

/** Legacy prefixed prop -> the component part it styles, and where. */
export const LEGACY_PART_PROPS: Readonly<
  Record<string, readonly [string, StyleGroup, string]>
> = {
  label_size: ['label', 'typography', 'size'],
  label_weight: ['label', 'typography', 'weight'],
  label_color: ['label', 'typography', 'color'],
  // Emitted as the label's bottom margin by the component expansion.
  label_spacing: ['label', 'spacing', 'margin_bottom'],
  field_background: ['field', 'fill', 'color'],
  field_border_color: ['field', 'stroke', 'color'],
  field_border_width: ['field', 'stroke', 'width'],
  field_radius: ['field', 'corners', 'radius'],
  // The expansion applies this as the field box's inner padding.
  field_padding: ['field', 'spacing', 'padding'],
}

function groupOf(style: NodeStyle | undefined, group: StyleGroup): Record<string, unknown> {
  return (style?.[group] as Record<string, unknown> | undefined) ?? {}
}

/**
 * Merges a node's style groups with its legacy props.
 *
 * The group wins when both are present (rule 6 of the spec) — a blueprint can
 * only reach that state by hand-editing, and the newer shape is the intent.
 */
export function resolveNodeStyle(node: Pick<ThemeNode, 'props' | 'style'>): ResolvedNodeStyle {
  const props = node.props ?? {}
  const resolved: Record<StyleGroup, Record<string, unknown>> = {
    fill: { ...groupOf(node.style, 'fill') },
    stroke: { ...groupOf(node.style, 'stroke') },
    corners: { ...groupOf(node.style, 'corners') },
    spacing: { ...groupOf(node.style, 'spacing') },
    typography: { ...groupOf(node.style, 'typography') },
  }

  for (const [legacyKey, [group, key]] of Object.entries(LEGACY_STYLE_PROPS)) {
    if (resolved[group][key] !== undefined) continue
    const value = props[legacyKey]
    if (value !== undefined && value !== '') {
      resolved[group][key] = value
    }
  }

  return resolved as unknown as ResolvedNodeStyle
}

/** A component part's style, with the legacy prefixed props folded in. */
export function resolvePartStyle(
  node: Pick<ThemeNode, 'props' | 'style'>,
  part: string,
): ResolvedNodeStyle {
  const props = node.props ?? {}
  const partStyle = node.style?.parts?.[part]
  const resolved = resolveNodeStyle({ style: partStyle })

  for (const [legacyKey, [legacyPart, group, key]] of Object.entries(LEGACY_PART_PROPS)) {
    if (legacyPart !== part) continue
    if ((resolved[group] as Record<string, unknown>)[key] !== undefined) continue
    const value = props[legacyKey]
    if (value !== undefined && value !== '') {
      ;(resolved[group] as Record<string, unknown>)[key] = value
    }
  }

  return resolved
}

function pruneEmptyGroups(style: NodeStyle): NodeStyle | undefined {
  const next: NodeStyle = {}
  let hasAny = false
  for (const group of STYLE_GROUPS) {
    const values = style[group] as Record<string, unknown> | undefined
    if (values && Object.keys(values).length > 0) {
      ;(next as Record<string, unknown>)[group] = values
      hasAny = true
    }
  }
  if (style.parts && Object.keys(style.parts).length > 0) {
    next.parts = style.parts
    hasAny = true
  }
  // Unknown groups are kept verbatim: round-tripping must not delete authored
  // data just because this version does not understand it.
  for (const [key, value] of Object.entries(style)) {
    if (key === 'parts') continue
    if ((STYLE_GROUPS as readonly string[]).includes(key)) continue
    ;(next as Record<string, unknown>)[key] = value
    hasAny = true
  }
  return hasAny ? next : undefined
}

/**
 * Moves a node's legacy styling props into style groups and drops them.
 *
 * Recurses through children and slots so one pass converts a whole page.
 * Non-styling props are untouched.
 */
export function normalizeNode(node: ThemeNode): ThemeNode {
  const props = { ...(node.props ?? {}) }
  const style: NodeStyle = { ...(node.style ?? {}) }
  let changed = false

  const assign = (group: StyleGroup, key: string, value: unknown) => {
    const current = { ...((style[group] as Record<string, unknown> | undefined) ?? {}) }
    if (current[key] === undefined) {
      current[key] = value
    }
    ;(style as Record<string, unknown>)[group] = current
  }

  for (const [legacyKey, [group, key]] of Object.entries(LEGACY_STYLE_PROPS)) {
    if (!(legacyKey in props)) continue
    const value = props[legacyKey]
    delete props[legacyKey]
    changed = true
    if (value === undefined || value === '') continue
    assign(group, key, value)
  }

  const parts: Record<string, NodeStyle> = { ...(style.parts ?? {}) }
  for (const [legacyKey, [part, group, key]] of Object.entries(LEGACY_PART_PROPS)) {
    if (!(legacyKey in props)) continue
    const value = props[legacyKey]
    delete props[legacyKey]
    changed = true
    if (value === undefined || value === '') continue
    const partStyle: NodeStyle = { ...(parts[part] ?? {}) }
    const current = { ...((partStyle[group] as Record<string, unknown> | undefined) ?? {}) }
    if (current[key] === undefined) current[key] = value
    ;(partStyle as Record<string, unknown>)[group] = current
    parts[part] = partStyle
  }
  if (Object.keys(parts).length > 0) {
    style.parts = parts
  }

  const children = node.children?.map(normalizeNode)
  const childrenChanged =
    children !== undefined && children.some((child, index) => child !== node.children?.[index])

  let slots: Record<string, ThemeNode> | undefined
  let slotsChanged = false
  if (node.slots && Object.keys(node.slots).length > 0) {
    slots = {}
    for (const [key, slotNode] of Object.entries(node.slots)) {
      const normalized = normalizeNode(slotNode)
      slots[key] = normalized
      if (normalized !== slotNode) slotsChanged = true
    }
  }

  if (!changed && !childrenChanged && !slotsChanged) {
    return node
  }

  const nextStyle = pruneEmptyGroups(style)
  const next: ThemeNode = {
    ...node,
    props,
    ...(children ? { children } : {}),
    ...(slots ? { slots } : {}),
  }
  if (nextStyle) {
    next.style = nextStyle
  } else {
    delete next.style
  }
  return next
}

export function normalizeNodes(nodes: ThemeNode[]): ThemeNode[] {
  const next = nodes.map(normalizeNode)
  return next.some((node, index) => node !== nodes[index]) ? next : nodes
}

/** Reads one style value, legacy props included. */
export function readStyleValue(
  node: Pick<ThemeNode, 'props' | 'style'>,
  group: StyleGroup,
  key: string,
): unknown {
  return (resolveNodeStyle(node)[group] as Record<string, unknown>)[key]
}

/**
 * Writes one style value, clearing it when the value is blank.
 *
 * The matching legacy prop is dropped at the same time, so an edited node
 * cannot end up carrying both.
 */
export function withStyleValue(
  node: ThemeNode,
  group: StyleGroup,
  key: string,
  value: unknown,
): ThemeNode {
  const style: NodeStyle = { ...(node.style ?? {}) }
  const current = { ...((style[group] as Record<string, unknown> | undefined) ?? {}) }
  if (value === undefined || value === '' || value === null) {
    delete current[key]
  } else {
    current[key] = value
  }
  ;(style as Record<string, unknown>)[group] = current

  const props = { ...(node.props ?? {}) }
  for (const [legacyKey, [legacyGroup, legacyField]] of Object.entries(LEGACY_STYLE_PROPS)) {
    if (legacyGroup === group && legacyField === key) {
      delete props[legacyKey]
    }
  }

  const next: ThemeNode = { ...node, props }
  const pruned = pruneEmptyGroups(style)
  if (pruned) {
    next.style = pruned
  } else {
    delete next.style
  }
  return next
}

/** Writes one style value onto a composed component's part. */
export function withPartStyleValue(
  node: ThemeNode,
  part: string,
  group: StyleGroup,
  key: string,
  value: unknown,
): ThemeNode {
  const parts: Record<string, NodeStyle> = { ...(node.style?.parts ?? {}) }
  const partStyle: NodeStyle = { ...(parts[part] ?? {}) }
  const current = { ...((partStyle[group] as Record<string, unknown> | undefined) ?? {}) }
  if (value === undefined || value === '' || value === null) {
    delete current[key]
  } else {
    current[key] = value
  }
  ;(partStyle as Record<string, unknown>)[group] = current

  const pruned = pruneEmptyGroups(partStyle)
  if (pruned) {
    parts[part] = pruned
  } else {
    delete parts[part]
  }

  const props = { ...(node.props ?? {}) }
  for (const [legacyKey, [legacyPart, legacyGroup, legacyField]] of Object.entries(
    LEGACY_PART_PROPS,
  )) {
    if (legacyPart === part && legacyGroup === group && legacyField === key) {
      delete props[legacyKey]
    }
  }

  const style: NodeStyle = { ...(node.style ?? {}) }
  if (Object.keys(parts).length > 0) {
    style.parts = parts
  } else {
    delete style.parts
  }

  const next: ThemeNode = { ...node, props }
  const nextStyle = pruneEmptyGroups(style)
  if (nextStyle) {
    next.style = nextStyle
  } else {
    delete next.style
  }
  return next
}

/** One inspector edit, applied by the builder page against the real node. */
export interface StyleEdit {
  group: StyleGroup
  key: string
  /** Names a composed component's part when the edit targets one. */
  part?: string
  value: unknown
}

export function applyStyleEdit(node: ThemeNode, edit: StyleEdit): ThemeNode {
  return edit.part
    ? withPartStyleValue(node, edit.part, edit.group, edit.key, edit.value)
    : withStyleValue(node, edit.group, edit.key, edit.value)
}
