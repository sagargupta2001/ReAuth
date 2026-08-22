import { Contrast, CornerUpLeft, Type } from 'lucide-react'

import {
  SettingsFieldKind,
  type ColorContrastPair,
  type SettingsSection,
} from '@/features/fluid/model/settingsFields'
import {
  ColorToken,
  RadiusToken,
  TOKEN_FALLBACK,
  TokenGroup,
  TypographyToken,
} from '@/features/fluid/model/tokens'

const textRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Text,
  fallback: TOKEN_FALLBACK.text,
} as const
const backgroundRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Background,
  fallback: TOKEN_FALLBACK.background,
} as const
const surfaceRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Surface,
  fallback: TOKEN_FALLBACK.surface,
} as const
const primaryRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Primary,
  fallback: TOKEN_FALLBACK.primary,
} as const

/**
 * The colour pairs the renderers actually paint.
 *
 * Primary buttons render white label text on the primary colour (see the
 * `Button` case in `FluidCanvas` / `FluidLoginScreen`), which is why that pair's
 * foreground is a literal rather than a token.
 */
const CONTRAST_PAIRS: readonly ColorContrastPair[] = [
  {
    label: 'Text on background',
    foreground: textRef,
    background: backgroundRef,
    minRatio: 4.5,
  },
  {
    label: 'Text on surface',
    foreground: textRef,
    background: surfaceRef,
    minRatio: 4.5,
  },
  {
    label: 'Primary button label',
    foreground: { literal: '#ffffff' },
    background: primaryRef,
    minRatio: 4.5,
  },
  {
    label: 'Links on surface',
    foreground: primaryRef,
    background: surfaceRef,
    minRatio: 4.5,
  },
]

/**
 * Declarative description of the theme settings panel.
 *
 * The panel renders whatever is listed here, so a new theme property is a new
 * entry in this file — no component changes required.
 *
 * Every token below is read by both renderers (`FluidCanvas` for the builder
 * preview and `FluidLoginScreen` at runtime). Keep it that way: a control for a
 * token nothing reads is a control that silently does nothing.
 */
export const THEME_SETTINGS_SECTIONS: readonly SettingsSection[] = [
  {
    id: 'layout',
    title: 'Layout',
    description: 'Choose the structural shell.',
    fields: [{ kind: SettingsFieldKind.LayoutShell, id: 'layout-shell' }],
  },
  {
    id: 'assets',
    title: 'Assets',
    description: 'Upload images or fonts for this theme.',
    fields: [{ kind: SettingsFieldKind.Assets, id: 'theme-assets' }],
  },
  {
    id: 'colors',
    title: 'Colors',
    description: 'Global palette overrides.',
    fields: [
      {
        kind: SettingsFieldKind.Color,
        id: 'primary',
        label: 'Primary',
        group: TokenGroup.Colors,
        token: ColorToken.Primary,
        fallback: TOKEN_FALLBACK.primary,
      },
      {
        kind: SettingsFieldKind.Color,
        id: 'background',
        label: 'Background',
        group: TokenGroup.Colors,
        token: ColorToken.Background,
        fallback: TOKEN_FALLBACK.background,
      },
      {
        kind: SettingsFieldKind.Color,
        id: 'text',
        label: 'Text',
        group: TokenGroup.Colors,
        token: ColorToken.Text,
        fallback: TOKEN_FALLBACK.text,
      },
      {
        kind: SettingsFieldKind.Color,
        id: 'surface',
        label: 'Surface',
        group: TokenGroup.Colors,
        token: ColorToken.Surface,
        fallback: TOKEN_FALLBACK.surface,
      },
      {
        kind: SettingsFieldKind.ColorContrast,
        id: 'color-contrast',
        label: 'Contrast',
        icon: Contrast,
        hint: 'WCAG AA needs 4.5:1 for text and 3:1 for large text and UI.',
        pairs: CONTRAST_PAIRS,
      },
    ],
  },
  {
    id: 'typography',
    title: 'Typography',
    description: 'Global font tokens.',
    fields: [
      {
        kind: SettingsFieldKind.Text,
        id: 'font-family',
        label: 'Font Family',
        icon: Type,
        group: TokenGroup.Typography,
        token: TypographyToken.FontFamily,
        placeholder: 'system-ui',
      },
      {
        kind: SettingsFieldKind.Number,
        id: 'base-size',
        label: 'Base Size',
        hint: 'Root font size in pixels. Every relative size scales from this.',
        group: TokenGroup.Typography,
        token: TypographyToken.BaseSize,
        placeholder: '16',
        min: 10,
        max: 32,
        step: 1,
      },
    ],
  },
  {
    id: 'effects',
    title: 'Effects',
    description: 'Corner rounding applied across the theme.',
    fields: [
      {
        kind: SettingsFieldKind.Number,
        id: 'radius-base',
        label: 'Radius',
        icon: CornerUpLeft,
        hint: 'Base corner radius in pixels.',
        group: TokenGroup.Radius,
        token: RadiusToken.Base,
        placeholder: '8',
        min: 0,
        max: 40,
        step: 1,
      },
    ],
  },
]
