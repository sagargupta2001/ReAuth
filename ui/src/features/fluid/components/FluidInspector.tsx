import { useEffect, useMemo, useState } from 'react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Textarea } from '@/components/textarea'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import { createNodeFromDefinition } from '@/features/fluid/lib/nodeUtils'
import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'
import { SectionCard } from '@/shared/ui/section-card'
import { Label } from '@/shared/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { contrastRatio } from '@/shared/lib/colorUtils'
import { ActionsPanel } from './inspector/ActionsPanel'
import { ContrastCard } from './inspector/ContrastCard'
import { IconPicker } from './inspector/IconPicker'
import {
  createActionId,
  type InspectorAction,
} from '@/features/fluid/lib/actionBindings'
import { TypographyControls } from './inspector/TypographyControls'
import { SpacingControls } from './inspector/SpacingControls'

interface FluidInspectorProps {
  assets: ThemeAsset[]
  tokens: Record<string, unknown>
  selectedBlock: ThemeNode | null
  validationErrors?: ThemeValidationError[]
  inputNames: string[]
  onUpdateSelectedBlock: (partial: {
    props?: Record<string, unknown>
    layout?: Record<string, unknown>
    size?: Record<string, unknown>
    slots?: Record<string, ThemeNode | null>
  }) => void
}

export function FluidInspector({
  assets,
  tokens,
  selectedBlock,
  validationErrors = [],
  inputNames,
  onUpdateSelectedBlock,
}: FluidInspectorProps) {
  const selectedProps = useMemo(() => selectedBlock?.props ?? {}, [selectedBlock])
  const selectedLayout = useMemo(() => selectedBlock?.layout ?? {}, [selectedBlock])
  const selectedSize = useMemo(() => selectedBlock?.size ?? {}, [selectedBlock])
  const selectedType = selectedBlock?.type ?? ''
  const selectedComponent = selectedBlock?.component ?? ''
  const displayType =
    selectedType === 'Component' && selectedComponent ? selectedComponent : selectedType
  const prefixSlot = selectedBlock?.slots?.prefix
  const errorSlot = selectedBlock?.slots?.error
  const colorTokens = useMemo(() => {
    const raw = tokens?.colors
    if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
      return raw as Record<string, unknown>
    }
    return {}
  }, [tokens])
  const baseBackground = String(colorTokens.background || colorTokens.surface || '#ffffff')
  const baseText = String(colorTokens.text || '#111827')
  const textColor = String(selectedProps.color || baseText)
  const textContrast = useMemo(
    () => contrastRatio(textColor, baseBackground),
    [textColor, baseBackground],
  )
  const selectedErrors = useMemo(
    () =>
      selectedBlock
        ? validationErrors.filter((error) => error.nodeId === selectedBlock.id)
        : [],
    [selectedBlock, validationErrors],
  )
  const contextSuggestions = useMemo(
    () => [
      'context.username',
      'context.email',
      'context.client_id',
      'context.realm',
      'context.resume_token',
      'context.action_type',
    ],
    [],
  )
  const supportsActions = useMemo(() => {
    if (!selectedBlock) return false
    if (selectedType === 'Input') return true
    if (selectedType === 'Component') {
      const component = selectedComponent.toLowerCase()
      return component === 'button' || component === 'input'
    }
    return false
  }, [selectedBlock, selectedType, selectedComponent])
  const [activeTab, setActiveTab] = useState<'properties' | 'actions'>('properties')
  const resolvedTab = supportsActions ? activeTab : 'properties'

  const rawActions = useMemo<InspectorAction[]>(() => {
    if (!selectedBlock) return []
    const rawActions = (selectedProps as Record<string, unknown>).actions
    if (!Array.isArray(rawActions)) return []
    return rawActions.filter((action) => action && typeof action === 'object') as InspectorAction[]
  }, [selectedBlock, selectedProps])
  const actions = useMemo(
    () =>
      rawActions.map((action) => ({
        ...action,
        action_id: action.action_id ?? createActionId(),
      })),
    [rawActions],
  )

  const updateActions = (nextActions: InspectorAction[]) => {
    if (!selectedBlock) return
    const nextProps = { ...(selectedProps as Record<string, unknown>) }
    nextProps.actions = nextActions.map((action) => ({
      ...action,
      action_id: action.action_id ?? createActionId(),
    }))
    onUpdateSelectedBlock({ props: nextProps })
  }

  useEffect(() => {
    if (!selectedBlock) return
    if (rawActions.length === 0) return
    const missing = rawActions.some((action) => !action.action_id)
    if (!missing) return

    const nextProps = { ...(selectedProps as Record<string, unknown>) }
    nextProps.actions = actions.map((action) => ({
      ...action,
      action_id: action.action_id ?? createActionId(),
    }))
    onUpdateSelectedBlock({ props: nextProps })
  }, [selectedBlock, rawActions, actions, selectedProps, onUpdateSelectedBlock])

  const upsertSlot = (
    key: string,
    baseType: ThemeNode['type'],
    props: Record<string, unknown>,
  ) => {
    const baseNode =
      selectedBlock?.slots?.[key] ??
      createNodeFromDefinition({
        type: baseType,
        props,
      })
    const updated = {
      ...baseNode,
      props: {
        ...(baseNode.props ?? {}),
        ...props,
      },
    }
    onUpdateSelectedBlock({ slots: { [key]: updated } })
  }

  return (
    <aside className="bg-muted/10 flex w-80 flex-col border-l">
      <div className="bg-background border-b px-4 py-3">
        <h3 className="text-sm font-semibold">Inspector</h3>
      </div>

      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        <SectionCard title="Element" description="Selected block properties.">
            {!selectedBlock ? (
              <p className="text-muted-foreground text-sm">
                Select a block from the canvas to edit its properties.
              </p>
            ) : (
              <>
                {selectedErrors.length > 0 && (
                  <div className="border-destructive/40 bg-destructive/10 text-destructive rounded-md border p-2 text-[11px]">
                    <div className="font-semibold">Validation</div>
                    <div className="mt-1 space-y-1">
                      {selectedErrors.map((error, index) => (
                        <div key={`${error.path}-${index}`}>{error.message}</div>
                      ))}
                    </div>
                  </div>
                )}
                {supportsActions && (
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      variant={resolvedTab === 'properties' ? 'default' : 'outline'}
                      className="h-8 px-3 text-xs"
                      onClick={() => setActiveTab('properties')}
                    >
                      Properties
                    </Button>
                    <Button
                      size="sm"
                      variant={resolvedTab === 'actions' ? 'default' : 'outline'}
                      className="h-8 px-3 text-xs"
                      onClick={() => setActiveTab('actions')}
                    >
                      Actions
                    </Button>
                  </div>
                )}
                {resolvedTab === 'properties' ? (
                  <>
                    <div className="space-y-2">
                      <Label>Block Type</Label>
                      <Input value={displayType || 'Node'} disabled />
                    </div>

                {selectedType === 'Text' && (
                  <div className="space-y-2">
                    <Label htmlFor="text">Text</Label>
                    <Input
                      id="text"
                      value={String(selectedProps.text || '')}
                      onChange={(event) =>
                        onUpdateSelectedBlock({ props: { text: event.target.value } })
                      }
                    />
                  </div>
                )}

                {selectedType === 'Icon' && (
                  <div className="space-y-2">
                    <Label htmlFor="icon-name">Icon Name</Label>
                    <div className="flex gap-2">
                      <Input
                        id="icon-name"
                        value={String(selectedProps.name || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { name: event.target.value } })
                        }
                      />
                      <IconPicker
                        value={String(selectedProps.name || '')}
                        color={String(selectedProps.color || '')}
                        onSelect={(name) => onUpdateSelectedBlock({ props: { name } })}
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <Input
                        value={String(selectedProps.size || '')}
                        placeholder="Size"
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { size: event.target.value } })
                        }
                      />
                      <Input
                        value={String(selectedProps.color || '')}
                        placeholder="Color"
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { color: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Custom SVG</Label>
                      <Textarea
                        value={String(selectedProps.svg_path || '')}
                        placeholder="SVG path (d attribute)"
                        className="min-h-[80px] text-xs"
                        onChange={(event) =>
                          onUpdateSelectedBlock({
                            props: { svg_path: event.target.value },
                          })
                        }
                      />
                      <Input
                        value={String(selectedProps.svg_viewbox || '')}
                        placeholder="ViewBox (e.g. 0 0 24 24)"
                        onChange={(event) =>
                          onUpdateSelectedBlock({
                            props: { svg_viewbox: event.target.value },
                          })
                        }
                      />
                      <p className="text-muted-foreground text-[10px]">
                        When SVG path is provided, it overrides the icon name.
                      </p>
                    </div>
                  </div>
                )}

                {selectedType === 'Component' && selectedComponent === 'Input' && (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="label">Label</Label>
                      <Input
                        id="label"
                        value={String(selectedProps.label || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { label: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="name">Field Name</Label>
                      <Input
                        id="name"
                        value={String(selectedProps.name || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { name: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Input Type</Label>
                      <Select
                        value={String(selectedProps.input_type || 'text')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ props: { input_type: value } })
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
                    <div className="space-y-2">
                      <Label>Label Styling</Label>
                      <div className="grid grid-cols-2 gap-2">
                        <Input
                          value={String(selectedProps.label_size || '')}
                          placeholder="Size (e.g. 12px)"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { label_size: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.label_weight || '')}
                          placeholder="Weight (e.g. 600)"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { label_weight: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.label_color || '')}
                          placeholder="Color"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { label_color: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.label_spacing || '')}
                          placeholder="Spacing (px)"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { label_spacing: event.target.value },
                            })
                          }
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>Field Container</Label>
                      <div className="grid grid-cols-2 gap-2">
                        <Input
                          value={String(selectedProps.field_border_color || '')}
                          placeholder="Border color"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { field_border_color: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.field_border_width || '')}
                          placeholder="Border width"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { field_border_width: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.field_radius || '')}
                          placeholder="Radius"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { field_radius: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.field_background || '')}
                          placeholder="Background"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { field_background: event.target.value },
                            })
                          }
                        />
                        <Input
                          value={String(selectedProps.field_padding || '')}
                          placeholder="Padding"
                          onChange={(event) =>
                            onUpdateSelectedBlock({
                              props: { field_padding: event.target.value },
                            })
                          }
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>Prefix Icon</Label>
                      <Select
                        value={
                          prefixSlot && (prefixSlot.props?.visible ?? true) !== false
                            ? 'show'
                            : 'hide'
                        }
                        onValueChange={(value) =>
                          upsertSlot('prefix', 'Icon', { visible: value === 'show' })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Toggle prefix icon" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="show">Show</SelectItem>
                          <SelectItem value="hide">Hide</SelectItem>
                        </SelectContent>
                      </Select>
                      <div className="space-y-2">
                        <div className="flex gap-2">
                          <Input
                            value={String(prefixSlot?.props?.name || '')}
                            placeholder="Icon name"
                            onChange={(event) =>
                              upsertSlot('prefix', 'Icon', {
                                name: event.target.value,
                                visible: true,
                              })
                            }
                          />
                          <IconPicker
                            value={String(prefixSlot?.props?.name || '')}
                            color={String(prefixSlot?.props?.color || '')}
                            onSelect={(name) =>
                              upsertSlot('prefix', 'Icon', {
                                name,
                                visible: true,
                              })
                            }
                          />
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                          <Input
                            value={String(prefixSlot?.props?.size || '')}
                            placeholder="Size"
                            onChange={(event) =>
                              upsertSlot('prefix', 'Icon', {
                                size: event.target.value,
                                visible: true,
                              })
                            }
                          />
                          <Input
                            value={String(prefixSlot?.props?.color || '')}
                            placeholder="Color"
                            onChange={(event) =>
                              upsertSlot('prefix', 'Icon', {
                                color: event.target.value,
                                visible: true,
                              })
                            }
                          />
                        </div>
                        <Textarea
                          value={String(prefixSlot?.props?.svg_path || '')}
                          placeholder="SVG path (d attribute)"
                          className="min-h-[80px] text-xs"
                          onChange={(event) =>
                            upsertSlot('prefix', 'Icon', {
                              svg_path: event.target.value,
                              visible: true,
                            })
                          }
                        />
                        <Input
                          value={String(prefixSlot?.props?.svg_viewbox || '')}
                          placeholder="ViewBox (e.g. 0 0 24 24)"
                          onChange={(event) =>
                            upsertSlot('prefix', 'Icon', {
                              svg_viewbox: event.target.value,
                              visible: true,
                            })
                          }
                        />
                        <p className="text-muted-foreground text-[10px]">
                          When SVG path is provided, it overrides the icon name.
                        </p>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>Error Hint</Label>
                      <Select
                        value={
                          errorSlot && (errorSlot.props?.visible ?? false)
                            ? 'show'
                            : 'hide'
                        }
                        onValueChange={(value) =>
                          upsertSlot('error', 'Text', { visible: value === 'show' })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Toggle error hint" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="show">Show</SelectItem>
                          <SelectItem value="hide">Hide</SelectItem>
                        </SelectContent>
                      </Select>
                      <div className="grid grid-cols-2 gap-2">
                        <Input
                          value={String(errorSlot?.props?.text || '')}
                          placeholder="Error text"
                          onChange={(event) =>
                            upsertSlot('error', 'Text', {
                              text: event.target.value,
                              visible: true,
                            })
                          }
                        />
                        <Input
                          value={String(errorSlot?.props?.color || '')}
                          placeholder="Color"
                          onChange={(event) =>
                            upsertSlot('error', 'Text', {
                              color: event.target.value,
                              visible: true,
                            })
                          }
                        />
                      </div>
                    </div>
                  </>
                )}

                {selectedType === 'Component' && selectedComponent === 'Button' && (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="button-label">Label</Label>
                      <Input
                        id="button-label"
                        value={String(selectedProps.label || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { label: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Variant</Label>
                      <Select
                        value={String(selectedProps.variant || 'primary')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ props: { variant: value } })
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

                {selectedType === 'Component' && selectedComponent === 'Link' && (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="link-label">Label</Label>
                      <Input
                        id="link-label"
                        value={String(selectedProps.label || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { label: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="link-href">Href</Label>
                      <Input
                        id="link-href"
                        value={String(selectedProps.href || '')}
                        onChange={(event) =>
                          onUpdateSelectedBlock({ props: { href: event.target.value } })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Target</Label>
                      <Select
                        value={String(selectedProps.target || '_self')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ props: { target: value } })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select target" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="_self">Same tab</SelectItem>
                          <SelectItem value="_blank">New tab</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </>
                )}

                {selectedType === 'Image' && (
                  <>
                    <div className="space-y-2">
                      <Label>Asset</Label>
                      <Select
                        value={String(selectedProps.asset_id || '')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ props: { asset_id: value } })
                        }
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
                          onUpdateSelectedBlock({ props: { alt: event.target.value } })
                        }
                      />
                    </div>
                  </>
                )}

                {selectedType === 'Component' && selectedComponent === 'Divider' && (
                  <p className="text-muted-foreground text-sm">
                    Divider blocks have no editable properties.
                  </p>
                )}

                {selectedType === 'Box' && (
                  <div className="space-y-3">
                    <div className="space-y-2">
                      <Label>Direction</Label>
                      <Select
                        value={String(selectedLayout.direction || 'column')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ layout: { direction: value } })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select direction" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="column">Column</SelectItem>
                          <SelectItem value="row">Row</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>Gap</Label>
                      <Input
                        value={String(selectedLayout.gap ?? '')}
                        placeholder="e.g. 12"
                        onChange={(event) =>
                          onUpdateSelectedBlock({
                            layout: { gap: Number.parseFloat(event.target.value) || 0 },
                          })
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Alignment</Label>
                      <Select
                        value={String(selectedLayout.align || 'stretch')}
                        onValueChange={(value) =>
                          onUpdateSelectedBlock({ layout: { align: value } })
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="Select alignment" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="start">Start</SelectItem>
                          <SelectItem value="center">Center</SelectItem>
                          <SelectItem value="end">End</SelectItem>
                          <SelectItem value="stretch">Stretch</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>Padding (top right bottom left)</Label>
                      <Input
                        value={
                          Array.isArray(selectedLayout.padding)
                            ? selectedLayout.padding.join(', ')
                            : ''
                        }
                        placeholder="e.g. 16, 16, 16, 16"
                        onChange={(event) => {
                          const parts = event.target.value
                            .split(',')
                            .map((part) => Number.parseFloat(part.trim()))
                            .filter((value) => !Number.isNaN(value))
                          if (parts.length === 4) {
                            onUpdateSelectedBlock({
                              layout: { padding: parts as [number, number, number, number] },
                            })
                          }
                        }}
                      />
                    </div>
                  </div>
                )}

                <div className="space-y-2 border-t pt-4">
                  <Label>Slot</Label>
                  <Select
                    value={String(selectedProps.slot || 'form')}
                    onValueChange={(value) =>
                      onUpdateSelectedBlock({ props: { slot: value } })
                    }
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
                    onValueChange={(value) =>
                      onUpdateSelectedBlock({ props: { align: value } })
                    }
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
                    value={String(selectedSize.width || 'fill')}
                    onValueChange={(value) => onUpdateSelectedBlock({ size: { width: value } })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select width" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="fill">Fill</SelectItem>
                      <SelectItem value="hug">Hug</SelectItem>
                      <SelectItem value="fixed">Fixed</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                {String(selectedSize.width || '') === 'fixed' && (
                  <div className="space-y-2">
                    <Label htmlFor="width-value">Custom Width</Label>
                    <Input
                      id="width-value"
                      value={String(selectedSize.width_value || '')}
                      placeholder="e.g. 240px"
                      onChange={(event) =>
                        onUpdateSelectedBlock({ size: { width_value: event.target.value } })
                      }
                    />
                  </div>
                )}
                <div className="space-y-2">
                  <Label>Height</Label>
                  <Select
                    value={String(selectedSize.height || 'hug')}
                    onValueChange={(value) => onUpdateSelectedBlock({ size: { height: value } })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select height" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="hug">Hug</SelectItem>
                      <SelectItem value="fill">Fill</SelectItem>
                      <SelectItem value="fixed">Fixed</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                {String(selectedSize.height || '') === 'fixed' && (
                  <div className="space-y-2">
                    <Label htmlFor="height-value">Custom Height</Label>
                    <Input
                      id="height-value"
                      value={String(selectedSize.height_value || '')}
                      placeholder="e.g. 240px"
                      onChange={(event) =>
                        onUpdateSelectedBlock({ size: { height_value: event.target.value } })
                      }
                    />
                  </div>
                )}

                <div className="space-y-2">
                  <Label>Size</Label>
                  <Select
                    value={String(selectedProps.size || 'md')}
                    onValueChange={(value) =>
                      onUpdateSelectedBlock({ props: { size: value } })
                    }
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
              </>
            ) : (
              <ActionsPanel
                actions={actions}
                inputNames={inputNames}
                contextSuggestions={contextSuggestions}
                onChange={updateActions}
              />
            )}
          </>
        )}
        </SectionCard>

        <TypographyControls
          fontSize={String(selectedProps.font_size || '')}
          fontWeight={String(selectedProps.font_weight || '')}
          color={String(selectedProps.color || '#111827')}
          disabled={!selectedBlock}
          onChange={({ font_size, font_weight, color }) => {
            const updates: Record<string, unknown> = {}
            if (font_size !== undefined) updates.font_size = font_size
            if (font_weight !== undefined) updates.font_weight = font_weight
            if (color !== undefined) updates.color = color
            onUpdateSelectedBlock({ props: updates })
          }}
        />

        {selectedType === 'Text' && <ContrastCard ratio={textContrast} />}

        <SpacingControls
          padding={String(selectedProps.padding || '')}
          marginTop={String(selectedProps.margin_top || '')}
          marginBottom={String(selectedProps.margin_bottom || '')}
          disabled={!selectedBlock}
          onChange={({ padding, margin_top, margin_bottom }) => {
            const updates: Record<string, unknown> = {}
            if (padding !== undefined) updates.padding = padding
            if (margin_top !== undefined) updates.margin_top = margin_top
            if (margin_bottom !== undefined) updates.margin_bottom = margin_bottom
            onUpdateSelectedBlock({ props: updates })
          }}
        />
      </div>
    </aside>
  )
}
