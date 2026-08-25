import type { CSSProperties } from 'react'

import type { ThemeNode } from '@/entities/theme/model/types'
import { resolveNodeStyle } from '@/features/fluid/lib/nodeStyle'

/**
 * Everything both renderers derive from a node before they draw it.
 *
 * `FluidCanvas` (builder preview) and `FluidLoginScreen` (runtime) each held a
 * byte-identical copy of this. Duplicated derivation is how the two drifted
 * apart: several fixes this cycle had to be applied twice, and the
 * `ProviderButtons` gap existed because one side was updated and the other was
 * not. The geometry and typography live here once; the two callers keep only
 * their genuinely different behaviour (selection vs. form wiring).
 */
export interface NodeVisuals {
  props: Record<string, unknown>
  /** Text alignment class. Must be applied to a block element to take effect. */
  alignClass: string
  /** Height + text size for the node's `size` prop (sm / md / lg). */
  sizeClass: string
  widthClass: string
  heightClass: string
  /** `w-full` only when the node fills; for children of a flex row. */
  fillWidthClass: string
  /** `h-full` when the node fills or is fixed. */
  fillHeightClass: string
  /**
   * Classes for the element that actually paints inside the wrapper.
   *
   * The wrapper carries the node's size, so a painted child has to fill it.
   * Without these a fixed height sized an invisible wrapper while the bordered
   * element stayed at content height, and "hug" never shrink-wrapped because the
   * painted element was hard-coded to `w-full`.
   */
  innerWidthClass: string
  innerHeightClass: string
  /** Inline style: spacing, explicit dimensions, and typography overrides. */
  style: CSSProperties
  /** Raw typography props, so callers can tell "set" from "defaulted". */
  fontSize: string
  fontWeight: string
  fontColor: string
  size: string
  /** Raw sizing mode/value; the Image node needs them to pick a height. */
  heightMode: string
  heightValue: string
}

function toNumber(value: unknown): number {
  return Number.parseFloat(String(value ?? '0')) || 0
}

export function computeNodeVisuals(node: ThemeNode): NodeVisuals {
  const props = node.props ?? {}
  // Grouped style first, legacy props as the fallback. Every stored blueprint
  // predates the groups, so the fallback is permanent, not transitional.
  const { typography, spacing } = resolveNodeStyle(node)

  const align = String(typography.align || 'left')
  const alignClass =
    align === 'center' ? 'text-center' : align === 'right' ? 'text-right' : 'text-left'

  const fontSize = String(typography.size || '')
  const fontWeight = String(typography.weight || '')
  const fontColor = String(typography.color || '')

  const marginTop = toNumber(spacing.margin_top)
  const marginBottom = toNumber(spacing.margin_bottom)
  const padding = toNumber(spacing.padding)

  const widthMode = String(node.size?.width || props.width || 'fill')
  // Coerced here so every consumer gets a valid CSS length: a bare "240" is
  // dropped by the browser, which read as the control doing nothing.
  const widthValue = resolveCssLength(node.size?.width_value ?? props.width_value)
  const heightMode = String(node.size?.height || props.height || 'hug')
  const heightValue = resolveCssLength(node.size?.height_value ?? props.height_value)
  const size = String(props.size || 'md')

  // Only emit spacing the node actually asks for. Writing `0px` inline beat the
  // container's `space-y-*` classes, which flattened the whole page's rhythm.
  const style: CSSProperties = {}
  if (marginTop) style.marginTop = `${marginTop}px`
  if (marginBottom) style.marginBottom = `${marginBottom}px`
  if (padding) style.padding = `${padding}px`

  const widthClass =
    widthMode === 'hug' || widthMode === 'auto'
      ? 'w-auto'
      : widthMode === 'fixed' || widthMode === 'custom'
        ? ''
        : 'w-full'
  const heightClass =
    heightMode === 'fill' ? 'h-full' : heightMode === 'fixed' ? '' : 'h-auto'
  const fillHeightClass = heightMode === 'fill' || heightMode === 'fixed' ? 'h-full' : ''
  const fillWidthClass = widthMode === 'fill' ? 'w-full' : ''
  // `hug` must shrink-wrap; every other mode fills the wrapper, whose own width
  // is either a utility class or the inline value set below.
  const innerWidthClass =
    widthMode === 'hug' || widthMode === 'auto' ? 'w-fit' : 'w-full'
  const innerHeightClass = fillHeightClass

  if ((widthMode === 'fixed' || widthMode === 'custom') && widthValue) {
    style.width = widthValue
  }
  if (heightMode === 'fixed' && heightValue) {
    style.height = heightValue
  }

  if (fontSize) style.fontSize = fontSize
  if (fontWeight) {
    const numeric = Number.parseInt(fontWeight, 10)
    style.fontWeight = Number.isNaN(numeric) ? fontWeight : numeric
  }
  if (fontColor) style.color = fontColor

  const sizeClass =
    size === 'sm' ? 'h-8 text-xs' : size === 'lg' ? 'h-11 text-base' : 'h-9 text-sm'

  return {
    props,
    alignClass,
    sizeClass,
    widthClass,
    heightClass,
    fillWidthClass,
    fillHeightClass,
    innerWidthClass,
    innerHeightClass,
    style,
    fontSize,
    fontWeight,
    fontColor,
    size,
    heightMode,
    heightValue,
  }
}

export interface NodeTextDisplay {
  text: string
  /**
   * True when the node has only a `text_path`, so what is shown is the binding
   * name rather than real copy. The builder has no context to resolve against.
   */
  isBinding: boolean
}

/** Fallback shown when a Text node carries neither copy nor a binding. */
export const TEXT_FALLBACK = 'Headline'

/**
 * Resolves what a Text node should display when no context is available.
 *
 * A literal `text` wins as the design-time preview; otherwise the binding is
 * surfaced as `{path}` so a context-bound node is visibly bound instead of
 * silently reading as an unconfigured "Headline".
 */
export function resolveDisplayText(props: Record<string, unknown>): NodeTextDisplay {
  const literal = String(props.text || '')
  if (literal) return { text: literal, isBinding: false }

  const path = String(props.text_path || '').trim()
  if (path) return { text: `{${path}}`, isBinding: true }

  return { text: TEXT_FALLBACK, isBinding: false }
}

/**
 * Interprets a `visible` prop, which may be a boolean, a string, or a number.
 *
 * The two renderers had diverged on the last line: the runtime coerced with
 * `Boolean(value)` while the builder returned `true`, so `visible: 0` hid the
 * node in production but showed it in the preview. The runtime's reading is the
 * correct one, and the builder now matches it.
 */
export function resolveVisibleFlag(value: unknown): boolean {
  if (value === undefined) return true
  if (typeof value === 'boolean') return value
  if (typeof value === 'string') return value.toLowerCase() !== 'false'
  return Boolean(value)
}

/**
 * Emits a CSS length for a builder-entered value.
 *
 * A bare number is not valid CSS — `width: 240` and `border-radius: 12` are both
 * dropped by the browser — so a unitless value gets `px`. This is why the fixed
 * width/height fields and the corner radius all looked inert when someone typed
 * a plain number.
 */
export function resolveCssLength(value: unknown): string {
  const raw = String(value ?? '').trim()
  return /^-?\d*\.?\d+$/.test(raw) ? `${raw}px` : raw
}

/** @deprecated Prefer {@link resolveCssLength}; kept for the radius call sites. */
export const resolveRadius = resolveCssLength
