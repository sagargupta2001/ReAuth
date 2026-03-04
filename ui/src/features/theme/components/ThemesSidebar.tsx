import { useMemo, useState } from 'react'

import { Palette, Plus, Search } from 'lucide-react'
import { NavLink } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Separator } from '@/components/separator'
import type { Theme } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useThemes } from '@/features/theme/api/useThemes'
import { CreateThemeDialog } from '@/features/theme/components/CreateThemeDialog'
import { cn } from '@/lib/utils'

export function ThemesSidebar() {
  const realm = useActiveRealm()
  const { data: themes, isLoading } = useThemes()

  const [search, setSearch] = useState('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const filteredThemes = useMemo(() => {
    if (!themes) return []
    const normalized = search.trim().toLowerCase()

    return themes
      .filter((theme) => {
        if (!normalized) return true
        return theme.name.toLowerCase().includes(normalized)
      })
      .sort((a, b) => a.name.localeCompare(b.name))
  }, [themes, search])

  if (isLoading || !realm) return null

  return (
    <div className="bg-muted/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      <div className="bg-background flex h-14 shrink-0 items-center justify-between border-b px-4">
        <h2 className="text-sm font-semibold tracking-tight">Themes</h2>
        <Badge variant="secondary" className="text-muted-foreground h-5 px-1.5 text-[10px]">
          {themes?.length ?? 0}
        </Badge>
      </div>

      <div className="space-y-3 p-3">
        <div className="relative">
          <Search className="text-muted-foreground/50 absolute top-2.5 left-2.5 h-4 w-4" />
          <Input
            placeholder="Find a theme..."
            className="bg-background h-9 pl-9 text-sm transition-shadow focus-visible:ring-1"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>

      <Separator />

      <div className="flex-1 overflow-y-auto p-2">
        {filteredThemes.length > 0 ? (
          <div className="space-y-0.5">
            {filteredThemes.map((theme) => (
              <ThemeItem key={theme.id} theme={theme} realmName={realm} />
            ))}
          </div>
        ) : (
          <div className="flex h-32 flex-col items-center justify-center gap-2 text-center">
            <div className="bg-muted rounded-full p-3">
              <Palette className="text-muted-foreground h-4 w-4" />
            </div>
            <p className="text-muted-foreground text-xs">No themes match your search.</p>
          </div>
        )}
      </div>

      <div className="bg-background mt-auto border-t p-3">
        <Button
          className="w-full justify-start gap-2"
          size="sm"
          variant="default"
          onClick={() => setIsCreateOpen(true)}
        >
          <Plus className="h-4 w-4" />
          Create New Theme
        </Button>
      </div>

      <CreateThemeDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} />
    </div>
  )
}

function ThemeItem({ theme, realmName }: { theme: Theme; realmName: string }) {
  return (
    <NavLink
      to={`/${realmName}/themes/${theme.id}`}
      className={({ isActive }) =>
        cn(
          'group flex items-start gap-3 rounded-md px-3 py-2.5 text-sm transition-all',
          'hover:bg-accent/50 hover:text-accent-foreground',
          isActive
            ? 'bg-sidebar-accent text-sidebar-accent-foreground ring-border font-medium shadow-sm ring-1'
            : 'text-muted-foreground',
        )
      }
    >
      <div className="bg-background mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-md border shadow-sm">
        <Palette className="h-3.5 w-3.5 text-muted-foreground/80" />
      </div>

      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center gap-2">
          <span className="text-foreground truncate leading-none">{theme.name}</span>
          {theme.is_system && (
            <Badge variant="secondary" className="h-4 px-1.5 text-[9px]">
              Default
            </Badge>
          )}
        </div>
        <span className="text-muted-foreground/60 mt-1.5 truncate text-[10px]">
          {theme.description || 'No description'}
        </span>
      </div>
    </NavLink>
  )
}
