import { createContext, useContext } from 'react'

import { DEFAULT_THEME } from '@/shared/config/theme.ts'

export type Theme = 'dark' | 'light' | 'system'
export type ResolvedTheme = Exclude<Theme, 'system'>

export type ThemeProviderState = {
  defaultTheme: Theme
  resolvedTheme: ResolvedTheme
  theme: Theme
  setTheme: (theme: Theme) => void
  resetTheme: () => void
}

export const initialState: ThemeProviderState = {
  defaultTheme: DEFAULT_THEME,
  resolvedTheme: 'light',
  theme: DEFAULT_THEME,
  setTheme: () => null,
  resetTheme: () => null,
}

export const ThemeContext = createContext<ThemeProviderState>(initialState)

export const useTheme = () => {
  const context = useContext(ThemeContext)

  if (!context) throw new Error('useTheme must be used within a ThemeProvider')

  return context
}
