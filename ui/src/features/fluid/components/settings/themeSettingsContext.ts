import { createContext, useContext } from 'react'

import type { ThemeAsset } from '@/entities/theme/model/types'
import type { TokenGroup } from '@/features/fluid/model/tokens'

/**
 * Everything a settings field needs to read and write draft state.
 *
 * Fields are rendered from a schema, so they cannot receive tailored props —
 * they pull what they need from here instead.
 */
export interface ThemeSettingsContextValue {
  tokens: Record<string, unknown>
  setToken: (group: TokenGroup, token: string, value: unknown) => void
  layoutShell: string
  onLayoutShellChange: (shell: string) => void
  assets: ThemeAsset[]
  onUploadAsset: (file: File) => void
  isUploading: boolean
}

const ThemeSettingsContext = createContext<ThemeSettingsContextValue | null>(null)

export const ThemeSettingsProvider = ThemeSettingsContext.Provider

export function useThemeSettings(): ThemeSettingsContextValue {
  const value = useContext(ThemeSettingsContext)
  if (!value) {
    throw new Error('useThemeSettings must be used inside <ThemeSettingsProvider>.')
  }
  return value
}
