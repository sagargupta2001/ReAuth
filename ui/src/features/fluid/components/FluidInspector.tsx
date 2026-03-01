import { Ruler, Type } from 'lucide-react'

import { Input } from '@/components/input'
import type { ThemeAsset } from '@/entities/theme/model/types'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Label } from '@/shared/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

interface FluidInspectorProps {
  assets: ThemeAsset[]
  selectedBlock: {
    block: string
    props?: Record<string, unknown>
  } | null
  onUpdateSelectedBlock: (partial: Record<string, unknown>) => void
}

export function FluidInspector({
  assets,
  selectedBlock,
  onUpdateSelectedBlock,
}: FluidInspectorProps) {
  const selectedProps = selectedBlock?.props ?? {}

  return (
    <aside className="bg-muted/10 flex w-72 flex-col border-l">
      <div className="bg-background border-b px-4 py-3">
        <h3 className="text-sm font-semibold">Inspector</h3>
      </div>

      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Element</CardTitle>
            <CardDescription>Selected block properties.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {!selectedBlock ? (
              <p className="text-muted-foreground text-sm">
                Select a block from the canvas to edit its properties.
              </p>
            ) : (
              <>
                <div className="space-y-2">
                  <Label>Block Type</Label>
                  <Input value={selectedBlock.block} disabled />
                </div>

                {selectedBlock.block === 'text' && (
                  <div className="space-y-2">
                    <Label htmlFor="text">Text</Label>
                    <Input
                      id="text"
                      value={String(selectedProps.text || '')}
                      onChange={(event) =>
                        onUpdateSelectedBlock({ text: event.target.value })
                      }
                    />
                  </div>
                )}

                {selectedBlock.block === 'input' && (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="label">Label</Label>
                      <Input
                        id="label"
                        value={String(selectedProps.label || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ label: event.target.value })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="name">Field Name</Label>
                      <Input
                        id="name"
                        value={String(selectedProps.name || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ name: event.target.value })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Input Type</Label>
                      <Select
                        value={String(selectedProps.input_type || 'text')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ input_type: value })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select input type" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="text">Text</SelectItem>
                          <SelectItem value="email">Email</SelectItem>
                          <SelectItem value="password">Password</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </>
                )}

                {selectedBlock.block === 'button' && (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="button-label">Label</Label>
                      <Input
                        id="button-label"
                        value={String(selectedProps.label || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ label: event.target.value })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Variant</Label>
                      <Select
                        value={String(selectedProps.variant || 'primary')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ variant: value })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select variant" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="primary">Primary</SelectItem>
                          <SelectItem value="secondary">Secondary</SelectItem>
                          <SelectItem value="outline">Outline</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </>
                )}

                {selectedBlock.block === 'image' && (
                  <>
                    <div className="space-y-2">
                      <Label>Asset</Label>
                      <Select
                        value={String(selectedProps.asset_id || '')}
                        onValueChange={(value) => onUpdateSelectedBlock({ asset_id: value })}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select asset" />
                        </SelectTrigger>
                        <SelectContent>
                          {assets.length === 0 ? (
                            <SelectItem value="none" disabled>
                              No assets uploaded
                            </SelectItem>
                          ) : (
                            assets.map((asset) => (
                              <SelectItem key={asset.id} value={asset.id}>
                                {asset.filename}
                              </SelectItem>
                            ))
                          )}
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="alt">Alt Text</Label>
                      <Input
                        id="alt"
                        value={String(selectedProps.alt || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ alt: event.target.value })
                        }
                      />
                    </div>
                  </>
                )}

                {selectedBlock.block === 'divider' && (
                  <p className="text-muted-foreground text-sm">
                    Divider blocks have no editable properties.
                  </p>
                )}

                <div className="space-y-2 border-t pt-4">
                  <Label>Slot</Label>
                  <Select
                    value={String(selectedProps.slot || 'form')}
                    onValueChange={(value) => onUpdateSelectedBlock({ slot: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select slot" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="form">Form</SelectItem>
                      <SelectItem value="brand">Brand</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label>Alignment</Label>
                  <Select
                    value={String(selectedProps.align || 'left')}
                    onValueChange={(value) => onUpdateSelectedBlock({ align: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select alignment" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="left">Left</SelectItem>
                      <SelectItem value="center">Center</SelectItem>
                      <SelectItem value="right">Right</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label>Width</Label>
                  <Select
                    value={String(selectedProps.width || 'full')}
                    onValueChange={(value) => onUpdateSelectedBlock({ width: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select width" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="full">Full</SelectItem>
                      <SelectItem value="auto">Auto</SelectItem>
                      <SelectItem value="custom">Custom</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                {String(selectedProps.width || 'full') === 'custom' && (
                  <div className="space-y-2">
                    <Label htmlFor="width-value">Custom Width</Label>
                    <Input
                      id="width-value"
                      value={String(selectedProps.width_value || '')}
                      placeholder="e.g. 240px"
                      onChange={(event) =>
                        onUpdateSelectedBlock({ width_value: event.target.value })
                      }
                    />
                  </div>
                )}

                <div className="space-y-2">
                  <Label>Size</Label>
                  <Select
                    value={String(selectedProps.size || 'md')}
                    onValueChange={(value) => onUpdateSelectedBlock({ size: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select size" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="sm">Small</SelectItem>
                      <SelectItem value="md">Medium</SelectItem>
                      <SelectItem value="lg">Large</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                {selectedBlock.block === 'image' && (
                  <div className="space-y-2">
                    <Label htmlFor="height-value">Custom Height</Label>
                    <Input
                      id="height-value"
                      value={String(selectedProps.height_value || '')}
                      placeholder="e.g. 200px"
                      onChange={(event) =>
                        onUpdateSelectedBlock({ height_value: event.target.value })
                      }
                    />
                  </div>
                )}
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Typography</CardTitle>
            <CardDescription>Font overrides for this block.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Type className="h-3.5 w-3.5" />
              <span>Typography</span>
            </div>
            <div className="space-y-2">
              <Label htmlFor="font-size">Font Size</Label>
              <Input
                id="font-size"
                value={String(selectedProps.font_size || '')}
                placeholder="e.g. 16px"
                disabled={!selectedBlock}
                onChange={(event) =>
                  onUpdateSelectedBlock({ font_size: event.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="font-weight">Font Weight</Label>
              <Input
                id="font-weight"
                value={String(selectedProps.font_weight || '')}
                placeholder="e.g. 600 or bold"
                disabled={!selectedBlock}
                onChange={(event) =>
                  onUpdateSelectedBlock({ font_weight: event.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="font-color">Color</Label>
              <div className="flex items-center gap-2">
                <div
                  className="h-8 w-8 rounded-md border"
                  style={{ backgroundColor: String(selectedProps.color || '#111827') }}
                />
                <Input
                  id="font-color"
                  value={String(selectedProps.color || '')}
                  placeholder="#111827"
                  disabled={!selectedBlock}
                  onChange={(event) =>
                    onUpdateSelectedBlock({ color: event.target.value })
                  }
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Spacing</CardTitle>
            <CardDescription>Padding and margins.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Ruler className="h-3.5 w-3.5" />
              <span>Padding</span>
            </div>
            <Input
              value={String(selectedProps.padding || '')}
              disabled={!selectedBlock}
              onChange={(event) =>
                onUpdateSelectedBlock({ padding: event.target.value })
              }
            />
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Ruler className="h-3.5 w-3.5" />
              <span>Margin Top</span>
            </div>
            <Input
              value={String(selectedProps.margin_top || '')}
              disabled={!selectedBlock}
              onChange={(event) =>
                onUpdateSelectedBlock({ margin_top: event.target.value })
              }
            />
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Ruler className="h-3.5 w-3.5" />
              <span>Margin Bottom</span>
            </div>
            <Input
              value={String(selectedProps.margin_bottom || '')}
              disabled={!selectedBlock}
              onChange={(event) =>
                onUpdateSelectedBlock({ margin_bottom: event.target.value })
              }
            />
          </CardContent>
        </Card>
      </div>
    </aside>
  )
}
