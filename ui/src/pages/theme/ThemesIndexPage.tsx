import { useEffect, useMemo } from 'react'

import { Loader2, Palette } from 'lucide-react'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveTheme } from '@/features/theme/api/useActiveTheme'
import { useThemes } from '@/features/theme/api/useThemes'

export function ThemesIndexPage() {
  const navigate = useRealmNavigate()
  const { data: themes, isLoading } = useThemes()
  const { data: activeTheme, isLoading: isActiveLoading } = useActiveTheme()

  // Preselect a theme so the page never lands on an empty shell. Prefer the
  // realm's active/default theme, falling back to the first theme alphabetically.
  const defaultThemeId = useMemo(() => {
    if (activeTheme?.theme?.id) return activeTheme.theme.id
    if (!themes?.length) return undefined
    return [...themes].sort((a, b) => a.name.localeCompare(b.name))[0].id
  }, [activeTheme, themes])

  useEffect(() => {
    if (defaultThemeId) navigate(`/themes/${defaultThemeId}`, { replace: true })
  }, [defaultThemeId, navigate])

  if (isLoading || isActiveLoading || defaultThemeId) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Theme...</p>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col items-center justify-center space-y-4 text-center">
      <div className="bg-muted flex h-20 w-20 items-center justify-center rounded-full">
        <Palette className="text-muted-foreground h-10 w-10" />
      </div>
      <div className="max-w-md space-y-2">
        <h2 className="text-2xl font-bold tracking-tight">Themes</h2>
        <p className="text-muted-foreground">
          Themes control the visual experience for every authenticator node. Create your first theme
          from the sidebar to get started.
        </p>
      </div>
    </div>
  )
}
