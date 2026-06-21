import type { ReactNode } from 'react'

import { Copy, Palette, Sparkles } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import type { Theme } from '@/entities/theme/model/types'

interface ThemeHeaderProps {
  theme: Theme
  activeVersionNumber?: number | null
  actions?: ReactNode
}

export function ThemeHeader({ theme, activeVersionNumber, actions }: ThemeHeaderProps) {
  const navigate = useRealmNavigate()
  const hasActiveVersion = typeof activeVersionNumber === 'number'

  const copyId = () => {
    void navigator.clipboard
      .writeText(theme.id)
      .then(() => toast.success('Theme ID copied.'))
      .catch(() => toast.error('Failed to copy theme ID.'))
  }

  return (
    <header className="flex h-16 shrink-0 items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <Palette className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{theme.name}</h1>
            {theme.is_system && (
              <Badge variant="secondary" className="h-5 text-[10px]">
                Default
              </Badge>
            )}
          </div>
          <div className="text-muted-foreground flex items-center gap-1 text-xs">
            ID: <span className="font-mono opacity-70">{theme.id.slice(0, 8)}...</span>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-5 w-5 shrink-0"
              onClick={copyId}
              aria-label="Copy theme ID"
            >
              <Copy className="h-3 w-3" />
            </Button>
          </div>
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
              <span className="text-foreground font-semibold">v{activeVersionNumber}</span>
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

        {actions}

        <Button
          size="sm"
          onClick={() => navigate(`/themes/${theme.id}/fluid`)}
          className="gap-2"
          variant="secondary"
        >
          <Sparkles className="h-3.5 w-3.5" />
          Open in Fluid
        </Button>
      </div>
    </header>
  )
}
