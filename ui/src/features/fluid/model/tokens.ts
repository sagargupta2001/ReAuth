/**
 * Canonical names for every theme token group / key the Fluid builder can edit,
 * plus the option sets that go with them.
 *
 * Adding a new theme property starts here: give it a name in the right group,
 * then describe it in `themeSettingsSchema.ts`. Nothing else needs to change
 * unless the property needs a brand new control type.
 */

export const TokenGroup = {
  Colors: 'colors',
  Typography: 'typography',
  Radius: 'radius',
} as const

export type TokenGroup = (typeof TokenGroup)[keyof typeof TokenGroup]

export const ColorToken = {
  Primary: 'primary',
  Background: 'background',
  Text: 'text',
  Surface: 'surface',
} as const

export const TypographyToken = {
  FontFamily: 'font_family',
  BaseSize: 'base_size',
} as const

export const RadiusToken = {
  Base: 'base',
} as const

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
