import { MoreVertical, Palette, Sparkles } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import type { Theme } from '@/entities/theme/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

interface ThemeHeaderProps {
  theme: Theme
  activeVersionNumber?: number | null
}

export function ThemeHeader({ theme, activeVersionNumber }: ThemeHeaderProps) {
  const navigate = useRealmNavigate()
  const hasActiveVersion = typeof activeVersionNumber === 'number'

  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b px-6">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <Palette className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{theme.name}</h1>
            <Badge variant="outline" className="h-5 text-[10px]">
              Theme
            </Badge>
            {theme.is_system && (
              <Badge variant="secondary" className="h-5 text-[10px]">
                Default
              </Badge>
            )}
          </div>
          <span className="text-muted-foreground text-xs">
            ID: <span className="font-mono opacity-70">{theme.id.slice(0, 8)}...</span>
          </span>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <div className="text-muted-foreground mr-2 flex items-center gap-2 border-r px-3 text-xs">
          {hasActiveVersion ? (
            <>
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"></span>
                <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500"></span>
              </span>
              Active Version:{' '}
              <span className="text-foreground font-semibold">
                v{activeVersionNumber}
              </span>
            </>
          ) : (
            <>
              <span className="relative flex h-2 w-2">
                <span className="relative inline-flex h-2 w-2 rounded-full bg-yellow-500"></span>
              </span>
              Status: <span className="text-foreground font-semibold">Unpublished Draft</span>
            </>
          )}
        </div>

        <Button
          onClick={() => navigate(`/themes/${theme.id}/fluid`)}
          className="gap-2"
          variant="secondary"
        >
          <Sparkles className="h-3.5 w-3.5" />
          Open in Fluid
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
              <MoreVertical className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem>Duplicate</DropdownMenuItem>
            <DropdownMenuItem
              className="text-destructive"
              disabled={theme.is_system}
            >
              Archive
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
