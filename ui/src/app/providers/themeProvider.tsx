import { type ReactNode } from 'react'

import { ThemeContext, type Theme, type ResolvedTheme } from '@/shared/theme/ThemeContext'

type ThemeProviderProps = {
  children: ReactNode
}

export function ThemeProvider({
  children,
  ...props
}: ThemeProviderProps) {
  // Always dark theme now
  const resolvedTheme = 'dark' as ResolvedTheme

  const setTheme = () => {
    // No-op since we're strictly dark theme
  }

  const resetTheme = () => {
    // No-op
  }

  const contextValue = {
    defaultTheme: 'dark' as Theme,
    resolvedTheme,
    resetTheme,
    theme: 'dark' as Theme,
    setTheme,
  }

  return (
    <ThemeContext value={contextValue} {...props}>
      {children}
    </ThemeContext>
  )
}
