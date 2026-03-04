import { Check, ChevronDown, LayoutTemplate, Loader2, Palette, ShieldCheck } from 'lucide-react'
import { useMemo, useState } from 'react'

import type { Theme } from '@/entities/theme/model/types'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'
import { useThemePages } from '@/features/theme/api/useThemePages'
import { useThemePreview } from '@/features/theme/api/useThemePreview'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/shared/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/shared/ui/popover'

interface ThemeDetailsOverviewTabProps {
  theme: Theme
}

export function ThemeDetailsOverviewTab({ theme }: ThemeDetailsOverviewTabProps) {
  const { data: pages = [] } = useThemePages(theme.id)
  const [selectedPage, setSelectedPage] = useState<string>('login')
  const activePage = useMemo(() => {
    if (pages.length === 0) return undefined
    const direct = pages.find((page) => page.key === selectedPage)
    return direct ?? pages[0]
  }, [pages, selectedPage])
  const activePageKey = activePage?.key ?? selectedPage ?? 'login'
  const [open, setOpen] = useState(false)

  const { data: preview, isLoading } = useThemePreview(theme.id, { pageKey: activePageKey })

  const previewTokens = preview?.tokens ?? {
    colors: {
      primary: 'var(--primary)',
      background: 'var(--background)',
      text: 'var(--foreground)',
      surface: 'var(--card)',
    },
    appearance: {
      mode: 'auto',
    },
    radius: {
      base: 12,
    },
  }
  const previewLayout = preview?.layout ?? { shell: 'CenteredCard' }
  const previewNodes = useMemo(() => preview?.nodes ?? [], [preview])
  const previewAssets = preview?.assets ?? []

  return (
    <div className="grid h-full gap-6 p-6 lg:grid-cols-[2fr_1fr] lg:items-stretch">
      <Card className="flex h-full flex-col overflow-hidden">
        <CardHeader>
          <CardTitle>Preview</CardTitle>
          <CardDescription>
            A live preview of the login experience built with this theme.
          </CardDescription>
          <div className="mt-3">
            <Popover open={open} onOpenChange={setOpen}>
              <PopoverTrigger asChild>
                <Button variant="outline" size="sm" className="gap-2">
                  <span className="text-xs font-semibold">
                    {activePage?.label ?? activePageKey}
                  </span>
                  <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
                </Button>
              </PopoverTrigger>
              <PopoverContent align="start" className="w-64 p-0">
                <Command>
                  <CommandInput placeholder="Search pages..." />
                  <CommandList>
                    <CommandEmpty>No pages found.</CommandEmpty>
                    <CommandGroup>
                      {pages.map((page) => (
                        <CommandItem
                          key={page.key}
                          onSelect={() => {
                            setSelectedPage(page.key)
                            setOpen(false)
                          }}
                        >
                          <span className="flex flex-1 flex-col">
                            <span className="text-xs font-medium">{page.label}</span>
                            <span className="text-muted-foreground text-[10px]">
                              {page.description}
                            </span>
                          </span>
                          {page.key === activePageKey && (
                            <Check className="h-3.5 w-3.5 text-primary" />
                          )}
                        </CommandItem>
                      ))}
                    </CommandGroup>
                  </CommandList>
                </Command>
              </PopoverContent>
            </Popover>
          </div>
        </CardHeader>
        <CardContent className="flex-1">
          <div className="bg-muted/10 flex h-full  items-center justify-center rounded-lg border">
            {isLoading ? (
              <div className="text-muted-foreground flex items-center gap-2 text-sm">
                <Loader2 className="h-4 w-4 animate-spin" /> Loading preview...
              </div>
            ) : (
              <div className="h-full w-full">
                <FluidCanvas
                  tokens={previewTokens}
                  layout={previewLayout}
                  blocks={previewNodes}
                  assets={previewAssets}
                  selectedNodeId={null}
                  isInspecting={false}
                  showChrome={false}
                  onSelectNode={() => {}}
                />
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Theme Summary</CardTitle>
            <CardDescription>Quick reference for this theme.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="flex items-center gap-2">
              <LayoutTemplate className="text-muted-foreground h-4 w-4" />
              <span>Layout:</span>
              <span className="text-foreground font-medium">Centered Card</span>
            </div>
            <div className="flex items-center gap-2">
              <Palette className="text-muted-foreground h-4 w-4" />
              <span>Token Set:</span>
              <span className="text-foreground font-medium">Default Tokens</span>
            </div>
            <div className="flex items-center gap-2">
              <ShieldCheck className="text-muted-foreground h-4 w-4" />
              <span>Compliance:</span>
              <span className="text-foreground font-medium">Standard</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Description</CardTitle>
            <CardDescription>How this theme is intended to be used.</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              {theme.description || 'No description configured yet.'}
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
