/**
 * Canonical names for every theme token group / key the Fluid builder can edit,
 * plus the option sets that go with them.
 *
 * Adding a new theme property starts here: give it a name in the right group,
 * then describe it in `themeSettingsSchema.ts`. Nothing else needs to change
 * unless the property needs a brand new control type.
 */

export const TokenGroup = {
  Appearance: 'appearance',
  Colors: 'colors',
  Typography: 'typography',
  Radius: 'radius',
  Shadow: 'shadow',
  Spacing: 'spacing',
} as const

export type TokenGroup = (typeof TokenGroup)[keyof typeof TokenGroup]

export const AppearanceToken = {
  Mode: 'mode',
} as const

export const ColorToken = {
  Primary: 'primary',
  Background: 'background',
  Text: 'text',
  Surface: 'surface',
} as const

export const TypographyToken = {
  FontFamily: 'font_family',
} as const

export const RadiusToken = {
  Base: 'base',
} as const

export const ThemeMode = {
  Auto: 'auto',
  Light: 'light',
  Dark: 'dark',
} as const

export type ThemeMode = (typeof ThemeMode)[keyof typeof ThemeMode]

export interface TokenOption {
  value: string
  label: string
}

export const THEME_MODE_OPTIONS: readonly TokenOption[] = [
  { value: ThemeMode.Auto, label: 'Auto (System)' },
  { value: ThemeMode.Light, label: 'Light' },
  { value: ThemeMode.Dark, label: 'Dark' },
]

/** Fallbacks used when a token is missing from the draft. */
export const TOKEN_FALLBACK = {
  color: '#111827',
  primary: '#111827',
  background: '#F8FAFC',
  text: '#111827',
  surface: '#FFFFFF',
} as const

export const LayoutKey = {
  Shell: 'shell',
  Slots: 'slots',
} as const

export const DEFAULT_LAYOUT_SLOTS: readonly string[] = ['main']
