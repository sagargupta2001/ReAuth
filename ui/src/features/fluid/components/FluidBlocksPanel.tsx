import type { DragEvent } from 'react'
import { useMemo, useState } from 'react'

import { AlertCircle, GripVertical, Plus, Search, Trash2 } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Popover, PopoverAnchor, PopoverContent } from '@/components/popover'
import { Separator } from '@/components/separator'
import type { ThemeNode } from '@/entities/theme/model/types'
import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'
import { cn } from '@/lib/utils'
import {
  FLUID_BLOCKS,
  type FluidBlockDefinition,
  buildFluidNode,
} from '@/features/fluid/components/fluidBlocks'

interface FluidBlocksPanelProps {
  nodes: ThemeNode[]
  selectedNodeId: string | null
  validationErrors?: ThemeValidationError[]
  onSelectNode: (nodeId: string) => void
  onInsertNode: (node: ThemeNode, index: number) => void
  onRemoveNode: (nodeId: string) => void
  onReorderNodes: (fromIndex: number, toIndex: number) => void
}

const BLOCK_LABELS: Map<string, string> = new Map(
  FLUID_BLOCKS.map((block) => [
    block.node.component ?? block.node.type,
    block.label,
  ]),
)


export function FluidBlocksPanel({
  nodes,
  selectedNodeId,
  validationErrors = [],
  onSelectNode,
  onInsertNode,
  onRemoveNode,
  onReorderNodes,
}: FluidBlocksPanelProps) {
  const [query, setQuery] = useState('')
  const [openKey, setOpenKey] = useState<string | null>(null)
  const [insertIndex, setInsertIndex] = useState(0)
  const [hoveredBlockId, setHoveredBlockId] = useState<FluidBlockDefinition['id'] | null>(null)

  const filteredBlocks = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return FLUID_BLOCKS
    return FLUID_BLOCKS.filter((block) =>
      block.label.toLowerCase().includes(normalized),
    )
  }, [query])

  const groupedBlocks = useMemo(() => {
    const groups = new Map<string, FluidBlockDefinition[]>()
    filteredBlocks.forEach((block) => {
      const category = block.category || 'Blocks'
      const existing = groups.get(category) ?? []
      groups.set(category, [...existing, block])
    })
    return Array.from(groups.entries())
  }, [filteredBlocks])

  const previewBlock: FluidBlockDefinition =
    FLUID_BLOCKS.find((block) => block.id === hoveredBlockId) ?? FLUID_BLOCKS[0]

  const handleOpenPicker = (key: string, index: number) => {
    setInsertIndex(index)
    setOpenKey(key)
  }

  const handleSelectBlock = (blockId: FluidBlockDefinition['id']) => {
    const definition = FLUID_BLOCKS.find((block) => block.id === blockId)
    if (!definition) return
    onInsertNode(buildFluidNode(definition), insertIndex)
    setOpenKey(null)
  }

  const handleItemDragStart = (event: DragEvent<HTMLDivElement>, index: number) => {
    event.dataTransfer.setData('application/reauth-fluid-reorder', index.toString())
    event.dataTransfer.effectAllowed = 'move'
  }

  const handleItemDrop = (event: DragEvent<HTMLDivElement>, index: number) => {
    const payload = event.dataTransfer.getData('application/reauth-fluid-reorder')
    const fromIndex = Number.parseInt(payload, 10)
    if (Number.isNaN(fromIndex)) {
      return
    }

    event.preventDefault()
    event.stopPropagation()

    if (fromIndex === index) return
    onReorderNodes(fromIndex, index)
  }

  const handleItemDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }

  const pickerOpen = openKey !== null

  const errorMap = useMemo(() => {
    const map = new Map<string, ThemeValidationError[]>()
    validationErrors.forEach((error) => {
      if (!error.nodeId) return
      const existing = map.get(error.nodeId) ?? []
      map.set(error.nodeId, [...existing, error])
    })
    return map
  }, [validationErrors])
  const pageErrors = useMemo(
    () => validationErrors.filter((error) => !error.nodeId),
    [validationErrors],
  )

  const renderAnchorButton = (key: string, button: React.ReactElement) => {
    if (openKey === key) {
      return <PopoverAnchor asChild>{button}</PopoverAnchor>
    }
    return button
  }

  return (
    <Popover
      open={pickerOpen}
      onOpenChange={(open) => {
        if (!open) {
          setOpenKey(null)
        }
      }}
    >
      <aside className="bg-muted/10 flex w-[280px] flex-col border-r">
        <div className="bg-background flex items-center justify-between border-b px-4 py-3">
          <h3 className="text-sm font-semibold">Sections</h3>
          {renderAnchorButton(
            'header',
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={() => handleOpenPicker('header', nodes.length)}
            >
              <Plus className="h-4 w-4" />
            </Button>,
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          {pageErrors.length > 0 && (
            <div className="border-destructive/40 bg-destructive/10 text-destructive mb-3 rounded-md border px-2 py-1 text-[11px]">
              {pageErrors.length} page-level validation issue(s). Check the inspector.
            </div>
          )}
          <div className="space-y-3 text-xs">
            <TreeRow
              label="Page"
              depth={0}
              onAdd={() => handleOpenPicker('page', nodes.length)}
              renderButton={(button) => renderAnchorButton('page', button)}
            />
            <TreeRow
              label="Layout Container"
              depth={1}
              onAdd={() => handleOpenPicker('layout', nodes.length)}
              renderButton={(button) => renderAnchorButton('layout', button)}
            />
            <div className="space-y-1">
              {nodes.length === 0 && (
                <div className="text-muted-foreground pl-8 text-[11px]">
                  Add blocks to build this page.
                </div>
              )}
              {nodes.map((node, index) =>
                renderNodeRow({
                  node,
                  depth: 2,
                  index,
                  isRoot: true,
                  selectedNodeId,
                  errorMap,
                  onSelectNode,
                  onRemoveNode,
                  onOpenPicker: handleOpenPicker,
                  onDragStart: handleItemDragStart,
                  onDrop: handleItemDrop,
                  onDragOver: handleItemDragOver,
                }),
              )}
            </div>
          </div>
        </div>
        <Separator />
      </aside>

      <PopoverContent
        align="start"
        side="right"
        sideOffset={12}
        collisionPadding={16}
        className="w-[560px] p-0 data-[state=closed]:hidden"
      >
        <div className="flex">
          <div className="w-2/5 border-r p-4">
            <div className="relative">
              <Search className="text-muted-foreground/50 absolute left-2.5 top-2.5 h-4 w-4" />
              <Input
                placeholder="Search blocks..."
                className="bg-background h-9 pl-8 text-sm"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
              />
            </div>
            <div className="mt-4 max-h-[320px] space-y-4 overflow-y-auto pr-1">
              {groupedBlocks.map(([category, items]) => (
                <div key={category} className="space-y-2">
                  <span className="text-muted-foreground text-[10px] font-semibold uppercase tracking-wide">
                    {category}
                  </span>
                  <div className="space-y-1">
                    {items.map((block) => {
                      const Icon = block.icon
                      return (
                        <button
                          key={block.id}
                          type="button"
                          onMouseEnter={() => setHoveredBlockId(block.id)}
                          onFocus={() => setHoveredBlockId(block.id)}
                          onClick={() => handleSelectBlock(block.id)}
                          className="hover:bg-muted/40 flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-xs transition-colors"
                        >
                          <Icon className="text-muted-foreground h-3.5 w-3.5" />
                          <div className="flex flex-col">
                            <span className="text-[11px] font-semibold">{block.label}</span>
                            <span className="text-muted-foreground text-[10px]">
                              {block.description}
                            </span>
                          </div>
                        </button>
                      )
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="w-3/5 p-4">
            <div className="mb-3 text-xs font-semibold uppercase text-muted-foreground">
              Preview
            </div>
            <div className="rounded-lg border bg-background p-4 shadow-2xl">
              <div className="text-[11px] font-semibold">{previewBlock.label}</div>
              <p className="text-muted-foreground mb-4 text-[10px]">
                {previewBlock.description}
              </p>
              <BlockPreview blockId={previewBlock.id} />
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}

function TreeRow({
  label,
  depth,
  onAdd,
  renderButton,
}: {
  label: string
  depth: number
  onAdd: () => void
  renderButton: (button: React.ReactElement) => React.ReactElement
}) {
  const button = (
    <Button
      variant="ghost"
      size="icon"
      className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
      onClick={onAdd}
    >
      <Plus className="h-3.5 w-3.5" />
    </Button>
  )

  return (
    <div
      className={cn(
        'group flex items-center justify-between rounded-md px-2 py-1 text-[11px] font-semibold text-foreground/80',
        depth === 0 ? 'pl-2' : 'pl-6',
      )}
    >
      <span>{label}</span>
      {renderButton(button)}
    </div>
  )
}

function renderNodeRow({
  node,
  depth,
  index,
  isRoot,
  selectedNodeId,
  errorMap,
  onSelectNode,
  onRemoveNode,
  onOpenPicker,
  onDragStart,
  onDrop,
  onDragOver,
}: {
  node: ThemeNode
  depth: number
  index?: number
  isRoot: boolean
  selectedNodeId: string | null
  errorMap: Map<string, ThemeValidationError[]>
  onSelectNode: (nodeId: string) => void
  onRemoveNode: (nodeId: string) => void
  onOpenPicker: (key: string, index: number) => void
  onDragStart: (event: DragEvent<HTMLDivElement>, index: number) => void
  onDrop: (event: DragEvent<HTMLDivElement>, index: number) => void
  onDragOver: (event: DragEvent<HTMLDivElement>) => void
}) {
  const label =
    BLOCK_LABELS.get(node.component ?? node.type) ?? node.component ?? node.type
  const isSelected = selectedNodeId === node.id
  const errorCount = errorMap.get(node.id)?.length ?? 0
  const hasError = errorCount > 0
  const rowKey = `node-${node.id}`
  const row = (
    <div
      key={rowKey}
      className={cn(
        'group flex items-center gap-2 rounded-md py-1 pr-2 transition-colors',
        isSelected
          ? 'bg-primary/10 text-foreground'
          : hasError
            ? 'text-destructive/80 hover:bg-destructive/5'
            : 'text-muted-foreground hover:bg-muted/40',
      )}
      style={{ paddingLeft: `${depth * 12}px` }}
      draggable={isRoot}
      onDragStart={(event) => {
        if (isRoot && typeof index === 'number') {
          onDragStart(event, index)
        }
      }}
      onDrop={(event) => {
        if (isRoot && typeof index === 'number') {
          onDrop(event, index)
        }
      }}
      onDragOver={(event) => {
        if (isRoot) {
          onDragOver(event)
        }
      }}
    >
      {isRoot && <GripVertical className="h-3.5 w-3.5 text-muted-foreground/60" />}
      <button
        type="button"
        className="flex flex-1 items-center gap-2 text-left"
        onClick={() => onSelectNode(node.id)}
      >
        <span className="text-[11px] font-medium">{label}</span>
        {hasError && (
          <span
            className="text-destructive flex items-center gap-1 text-[10px] font-semibold"
            title={`${errorCount} validation issue(s)`}
          >
            <AlertCircle className="h-3 w-3" />
            {errorCount}
          </span>
        )}
      </button>
      {isRoot && typeof index === 'number' && (
        <>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
            onClick={() => onOpenPicker(rowKey, index + 1)}
          >
            <Plus className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
            onClick={() => onRemoveNode(node.id)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </>
      )}
      {!isRoot && (
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
          onClick={() => onRemoveNode(node.id)}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </Button>
      )}
    </div>
  )

  const childRows = (node.children ?? []).map((child) =>
    renderNodeRow({
      node: child,
      depth: depth + 1,
      isRoot: false,
      selectedNodeId,
      errorMap,
      onSelectNode,
      onRemoveNode,
      onOpenPicker,
      onDragStart,
      onDrop,
      onDragOver,
    }),
  )

  const slotRows = Object.entries(node.slots ?? {}).map(([slotKey, slotNode]) => (
    <div key={`${rowKey}-slot-${slotKey}`} className="space-y-1">
      <div
        className="text-muted-foreground/70 text-[10px] font-semibold uppercase"
        style={{ paddingLeft: `${(depth + 1) * 12}px` }}
      >
        Slot: {slotKey}
      </div>
      {renderNodeRow({
        node: slotNode,
        depth: depth + 2,
        isRoot: false,
        selectedNodeId,
        errorMap,
        onSelectNode,
        onRemoveNode,
        onOpenPicker,
        onDragStart,
        onDrop,
        onDragOver,
      })}
    </div>
  ))

  return (
    <div key={`${rowKey}-group`} className="space-y-1">
      {row}
      {slotRows}
      {childRows}
    </div>
  )
}

function BlockPreview({ blockId }: { blockId: FluidBlockDefinition['id'] }) {
  switch (blockId) {
    case 'box':
      return (
        <div className="space-y-2 rounded-md border border-dashed p-3 text-[10px] text-muted-foreground">
          <div className="h-3 w-20 rounded-full bg-muted/60" />
          <div className="h-3 w-24 rounded-full bg-muted/40" />
        </div>
      )
    case 'text':
      return <div className="text-lg font-semibold text-foreground">Welcome back</div>
    case 'input':
      return (
        <div className="space-y-2">
          <span className="text-muted-foreground text-[10px] uppercase tracking-wide">
            Email
          </span>
          <div className="h-9 w-full rounded-md border bg-muted/20" />
        </div>
      )
    case 'button':
      return (
        <div className="h-9 w-full rounded-md bg-primary text-center text-xs font-semibold text-primary-foreground">
          Continue
        </div>
      )
    case 'divider':
      return <div className="h-px w-full bg-border" />
    case 'link':
      return <div className="text-primary text-xs underline">Forgot password?</div>
    case 'image':
      return (
        <div className="flex h-28 w-full items-center justify-center rounded-md border bg-muted/20 text-[10px] text-muted-foreground">
          Image placeholder
        </div>
      )
    default:
      return null
  }
}
