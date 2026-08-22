import { Droplet, Sliders, Type } from 'lucide-react'

import {
  SettingsFieldKind,
  type SettingsSection,
} from '@/features/fluid/model/settingsFields'
import {
  AppearanceToken,
  ColorToken,
  RadiusToken,
  THEME_MODE_OPTIONS,
  ThemeMode,
  TOKEN_FALLBACK,
  TokenGroup,
  TypographyToken,
} from '@/features/fluid/model/tokens'

/**
 * Declarative description of the theme settings panel.
 *
 * The panel renders whatever is listed here, so a new theme property is a new
 * entry in this file — no component changes required.
 */
export const THEME_SETTINGS_SECTIONS: readonly SettingsSection[] = [
  {
    id: 'appearance',
    title: 'Appearance',
    description: 'Choose how the theme handles light/dark mode.',
    fields: [
      {
        kind: SettingsFieldKind.Select,
        id: 'appearance-mode',
        label: 'Theme Mode',
        group: TokenGroup.Appearance,
        token: AppearanceToken.Mode,
        options: THEME_MODE_OPTIONS,
        fallback: ThemeMode.Auto,
        placeholder: 'Select mode',
      },
    ],
  },
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
      },
    ],
  },
  {
    id: 'effects',
    title: 'Effects',
    description: 'Shadows and radius.',
    fields: [
      {
        kind: SettingsFieldKind.Text,
        id: 'radius-base',
        label: 'Radius',
        icon: Droplet,
        group: TokenGroup.Radius,
        token: RadiusToken.Base,
      },
      {
        kind: SettingsFieldKind.Static,
        id: 'shadow',
        label: 'Shadow',
        icon: Sliders,
        value: 'Soft',
      },
    ],
  },
]
