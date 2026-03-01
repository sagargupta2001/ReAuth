import { LayoutTemplate, Palette, ShieldCheck } from 'lucide-react'

import type { Theme } from '@/entities/theme/model/types'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'

interface ThemeDetailsOverviewTabProps {
  theme: Theme
}

export function ThemeDetailsOverviewTab({ theme }: ThemeDetailsOverviewTabProps) {
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
          <div className="bg-muted/10 flex min-h-[360px] items-center justify-center rounded-lg border p-8">
            <div className="bg-background w-full max-w-sm space-y-4 rounded-xl border p-6 shadow-sm">
              <div className="flex items-center gap-3">
                <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
                  <Palette className="text-primary h-5 w-5" />
                </div>
                <div>
                  <p className="text-sm font-semibold">{theme.name}</p>
                  <p className="text-muted-foreground text-xs">Universal Login</p>
                </div>
              </div>

              <div className="space-y-3">
                <div className="bg-muted/40 h-9 rounded-md" />
                <div className="bg-muted/40 h-9 rounded-md" />
                <div className="bg-primary/90 h-9 rounded-md" />
              </div>

              <div className="text-muted-foreground flex items-center justify-between text-[10px]">
                <span>Remember me</span>
                <span>Forgot password?</span>
              </div>
            </div>
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
