import type { ChangeEvent } from 'react'
import { useRef } from 'react'

import { Droplet, Image, Sliders, Type, UploadCloud } from 'lucide-react'

import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import type { ThemeAsset } from '@/entities/theme/model/types'
import { FluidLayoutGallery } from '@/features/fluid/components/FluidLayoutGallery'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Label } from '@/shared/ui/label'

interface FluidThemeSettingsPanelProps {
  tokens: Record<string, unknown>
  onTokensChange: (tokens: Record<string, unknown>) => void
  layout: Record<string, unknown>
  onLayoutChange: (layout: Record<string, unknown>) => void
  assets: ThemeAsset[]
  onUploadAsset: (file: File) => void
  isUploading?: boolean
}

function getNestedRecord(
  source: Record<string, unknown>,
  key: string,
): Record<string, unknown> {
  const value = source[key]
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as Record<string, unknown>
  }
  return {}
}

function normalizeColorValue(value: string) {
  const hex = value.trim()
  if (/^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(hex)) {
    return hex
  }
  return '#111827'
}

export function FluidThemeSettingsPanel({
  tokens,
  onTokensChange,
  layout,
  onLayoutChange,
  assets,
  onUploadAsset,
  isUploading,
}: FluidThemeSettingsPanelProps) {
  const fileInputRef = useRef<HTMLInputElement | null>(null)
  const colors = getNestedRecord(tokens, 'colors')
  const appearance = getNestedRecord(tokens, 'appearance')
  const typography = getNestedRecord(tokens, 'typography')
  const radius = getNestedRecord(tokens, 'radius')
  const currentShell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const themeMode = typeof appearance.mode === 'string' ? appearance.mode : 'auto'

  const updateTokens = (next: Record<string, unknown>) => {
    onTokensChange({
      ...tokens,
      ...next,
    })
  }

  const updateLayout = (shell: string) => {
    onLayoutChange({
      ...layout,
      shell,
      slots: Array.isArray(layout.slots) ? layout.slots : ['main'],
    })
  }

  const handleAssetSelect = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      onUploadAsset(file)
    }
    event.target.value = ''
  }

  return (
    <aside className="bg-muted/10 flex w-72 flex-col border-r">
      <div className="bg-background border-b px-4 py-3">
        <h3 className="text-sm font-semibold">Theme Settings</h3>
      </div>

      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Appearance</CardTitle>
            <CardDescription>Choose how the theme handles light/dark mode.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label className="text-xs">Theme Mode</Label>
            <Select
              value={themeMode}
              onValueChange={(value) =>
                updateTokens({
                  appearance: {
                    ...appearance,
                    mode: value,
                  },
                })
              }
            >
              <SelectTrigger className="bg-background h-8 text-xs">
                <SelectValue placeholder="Select mode" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">Auto (System)</SelectItem>
                <SelectItem value="light">Light</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
              </SelectContent>
            </Select>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Layout</CardTitle>
            <CardDescription>Choose the structural shell.</CardDescription>
          </CardHeader>
          <CardContent>
            <FluidLayoutGallery value={currentShell} onChange={updateLayout} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Assets</CardTitle>
            <CardDescription>Upload images or fonts for this theme.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <input
              ref={fileInputRef}
              type="file"
              className="hidden"
              onChange={handleAssetSelect}
            />
            <button
              type="button"
              className="border-border hover:border-primary/60 hover:bg-muted/40 flex w-full items-center justify-between rounded-lg border px-3 py-2 text-left text-xs transition-colors"
              onClick={() => fileInputRef.current?.click()}
            >
              <span className="flex items-center gap-2">
                <UploadCloud className="h-4 w-4 text-muted-foreground" />
                Upload asset
              </span>
              <span className="text-muted-foreground">
                {isUploading ? 'Uploading...' : 'PNG, JPG, SVG'}
              </span>
            </button>

            <div className="space-y-2">
              {assets.length === 0 ? (
                <p className="text-muted-foreground text-xs">No assets uploaded yet.</p>
              ) : (
                assets.map((asset) => (
                  <div
                    key={asset.id}
                    className="flex items-center gap-3 rounded-md border bg-background px-3 py-2 text-xs"
                  >
                    {asset.mime_type.startsWith('image/') ? (
                      <img
                        src={asset.url}
                        alt={asset.filename}
                        className="h-10 w-10 rounded-md border object-cover"
                      />
                    ) : (
                      <div className="bg-muted flex h-10 w-10 items-center justify-center rounded-md border">
                        <Image className="h-4 w-4 text-muted-foreground" />
                      </div>
                    )}
                    <div className="flex flex-1 flex-col">
                      <span className="font-medium">{asset.filename}</span>
                      <span className="text-muted-foreground text-[10px]">
                        {(asset.byte_size / 1024).toFixed(1)} KB · {asset.asset_type}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Colors</CardTitle>
            <CardDescription>Global palette overrides.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="primary">Primary</Label>
              <div className="flex items-center gap-2">
                <input
                  type="color"
                  aria-label="Primary color"
                  className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
                  value={normalizeColorValue(String(colors.primary || '#111827'))}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        primary: event.target.value,
                      },
                    })
                  }
                />
                <Input
                  id="primary"
                  value={String(colors.primary || '')}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        primary: event.target.value,
                      },
                    })
                  }
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="background">Background</Label>
              <div className="flex items-center gap-2">
                <input
                  type="color"
                  aria-label="Background color"
                  className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
                  value={normalizeColorValue(String(colors.background || '#F8FAFC'))}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        background: event.target.value,
                      },
                    })
                  }
                />
                <Input
                  id="background"
                  value={String(colors.background || '')}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        background: event.target.value,
                      },
                    })
                  }
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="text">Text</Label>
              <div className="flex items-center gap-2">
                <input
                  type="color"
                  aria-label="Text color"
                  className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
                  value={normalizeColorValue(String(colors.text || '#111827'))}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        text: event.target.value,
                      },
                    })
                  }
                />
                <Input
                  id="text"
                  value={String(colors.text || '')}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        text: event.target.value,
                      },
                    })
                  }
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="surface">Surface</Label>
              <div className="flex items-center gap-2">
                <input
                  type="color"
                  aria-label="Surface color"
                  className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
                  value={normalizeColorValue(String(colors.surface || '#FFFFFF'))}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        surface: event.target.value,
                      },
                    })
                  }
                />
                <Input
                  id="surface"
                  value={String(colors.surface || '')}
                  onChange={(event) =>
                    updateTokens({
                      colors: {
                        ...colors,
                        surface: event.target.value,
                      },
                    })
                  }
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Typography</CardTitle>
            <CardDescription>Global font tokens.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="flex items-center gap-2">
              <Type className="h-4 w-4 text-muted-foreground" />
              <span>Font Family</span>
            </div>
            <Input
              value={String(typography.font_family || '')}
              onChange={(event) =>
                updateTokens({
                  typography: {
                    ...typography,
                    font_family: event.target.value,
                  },
                })
              }
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Effects</CardTitle>
            <CardDescription>Shadows and radius.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="flex items-center gap-2">
              <Droplet className="h-4 w-4 text-muted-foreground" />
              <span>Radius</span>
            </div>
            <Input
              value={String(radius.base ?? '')}
              onChange={(event) =>
                updateTokens({
                  radius: {
                    ...radius,
                    base: event.target.value,
                  },
                })
              }
            />
            <div className="flex items-center gap-2">
              <Sliders className="h-4 w-4 text-muted-foreground" />
              <span>Shadow</span>
            </div>
            <Input value="Soft" disabled />
          </CardContent>
        </Card>
      </div>
    </aside>
  )
}
