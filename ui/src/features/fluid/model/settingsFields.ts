import type { LucideIcon } from 'lucide-react'

import type { TokenGroup } from '@/features/fluid/model/tokens'

/**
 * Every control type the theme settings panel knows how to render.
 *
 * To add one: add a member here, add its descriptor to `SettingsField`, then
 * render it in `ThemeSettingsField`. TypeScript's exhaustiveness check on the
 * discriminated union will point at the one place that still needs handling.
 */
export const SettingsFieldKind = {
  Color: 'color',
  Text: 'text',
  Number: 'number',
  /** Read-only WCAG contrast report over token pairs. */
  ColorContrast: 'color-contrast',
  LayoutShell: 'layout-shell',
  Assets: 'assets',
} as const

export type SettingsFieldKind = (typeof SettingsFieldKind)[keyof typeof SettingsFieldKind]

interface FieldBase {
  /** Stable identifier, also used as the control's DOM id. */
  id: string
  label?: string
  hint?: string
  icon?: LucideIcon
}

/** Fields that read from and write to a single token path. */
interface TokenFieldBase extends FieldBase {
  group: TokenGroup
  token: string
}

export interface ColorSettingsField extends TokenFieldBase {
  kind: typeof SettingsFieldKind.Color
  /** Swatch value used while the token is empty or invalid. */
  fallback: string
}

export interface TextSettingsField extends TokenFieldBase {
  kind: typeof SettingsFieldKind.Text
  placeholder?: string
}

export interface NumberSettingsField extends TokenFieldBase {
  kind: typeof SettingsFieldKind.Number
  placeholder?: string
  min?: number
  max?: number
  step?: number
}

/** One side of a contrast pair: either a theme token or a hard-coded colour. */
export type ColorRef =
  | { group: TokenGroup; token: string; fallback: string }
  | { literal: string }

export interface ColorContrastPair {
  /** What this pair represents on screen, e.g. "Body text on background". */
  label: string
  foreground: ColorRef
  background: ColorRef
  /** WCAG minimum for this pair (4.5 for normal text, 3 for large/UI). */
  minRatio: number
}

export interface ColorContrastSettingsField extends FieldBase {
  kind: typeof SettingsFieldKind.ColorContrast
  pairs: readonly ColorContrastPair[]
}

export interface LayoutShellSettingsField extends FieldBase {
  kind: typeof SettingsFieldKind.LayoutShell
}

export interface AssetsSettingsField extends FieldBase {
  kind: typeof SettingsFieldKind.Assets
}

export type SettingsField =
  | ColorSettingsField
  | TextSettingsField
  | NumberSettingsField
  | ColorContrastSettingsField
  | LayoutShellSettingsField
  | AssetsSettingsField

export interface SettingsSection {
  id: string
  title: string
  description: string
  fields: readonly SettingsField[]
}
