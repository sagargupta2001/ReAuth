import { LayoutTemplate, Loader2, Palette, ShieldCheck } from 'lucide-react'

import type { Theme } from '@/entities/theme/model/types'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'
import { useThemePreview } from '@/features/theme/api/useThemePreview'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'

interface ThemeDetailsOverviewTabProps {
  theme: Theme
}

export function ThemeDetailsOverviewTab({ theme }: ThemeDetailsOverviewTabProps) {
  const { data: preview, isLoading } = useThemePreview(theme.id, { pageKey: 'login' })

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
  const previewBlocks = preview?.blocks ?? []
  const previewAssets = preview?.assets ?? []

  return (
    <div className="grid gap-6 p-6 lg:grid-cols-[2fr_1fr]">
      <Card className="overflow-hidden">
        <CardHeader>
          <CardTitle>Preview</CardTitle>
          <CardDescription>
            A live preview of the login experience built with this theme.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-muted/10 flex min-h-[360px] items-center justify-center rounded-lg border">
            {isLoading ? (
              <div className="text-muted-foreground flex items-center gap-2 text-sm">
                <Loader2 className="h-4 w-4 animate-spin" /> Loading preview...
              </div>
            ) : (
              <div className="w-full">
                <FluidCanvas
                  tokens={previewTokens}
                  layout={previewLayout}
                  blocks={previewBlocks}
                  assets={previewAssets}
                  selectedIndex={null}
                  isInspecting={false}
                  showChrome={false}
                  onSelectBlock={() => {}}
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
