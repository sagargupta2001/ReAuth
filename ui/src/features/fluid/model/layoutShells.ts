import { LayoutTemplate, PanelRight, Square, type LucideIcon } from 'lucide-react'

/** Structural shells a theme page can be rendered inside. */
export const LayoutShell = {
  CenteredCard: 'CenteredCard',
  SplitScreen: 'SplitScreen',
  Minimal: 'Minimal',
} as const

export type LayoutShell = (typeof LayoutShell)[keyof typeof LayoutShell]

export const DEFAULT_LAYOUT_SHELL: LayoutShell = LayoutShell.CenteredCard

export interface LayoutShellOption {
  id: LayoutShell
  name: string
  description: string
  icon: LucideIcon
}

export const LAYOUT_SHELL_OPTIONS: readonly LayoutShellOption[] = [
  {
    id: LayoutShell.CenteredCard,
    name: 'Centered Card',
    description: 'Classic centered form layout.',
    icon: LayoutTemplate,
  },
  {
    id: LayoutShell.SplitScreen,
    name: 'Split Screen',
    description: 'Brand visual on the left, form on the right.',
    icon: PanelRight,
  },
  {
    id: LayoutShell.Minimal,
    name: 'Minimal',
    description: 'Simple edge-to-edge layout.',
    icon: Square,
  },
]
