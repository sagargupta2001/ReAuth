import type { LucideIcon } from 'lucide-react'

import type { TokenGroup, TokenOption } from '@/features/fluid/model/tokens'

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
  Select: 'select',
  /** Read-only placeholder for a token that is not editable yet. */
  Static: 'static',
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

export interface SelectSettingsField extends TokenFieldBase {
  kind: typeof SettingsFieldKind.Select
  options: readonly TokenOption[]
  fallback: string
  placeholder?: string
}

export interface StaticSettingsField extends FieldBase {
  kind: typeof SettingsFieldKind.Static
  value: string
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
  | SelectSettingsField
  | StaticSettingsField
  | LayoutShellSettingsField
  | AssetsSettingsField

export interface SettingsSection {
  id: string
  title: string
  description: string
  fields: readonly SettingsField[]
}
