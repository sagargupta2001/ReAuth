import type { DragEvent } from 'react'
import { useMemo, useState } from 'react'

import { GripVertical, Plus, Search, Trash2 } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Popover, PopoverAnchor, PopoverContent } from '@/components/popover'
import { Separator } from '@/components/separator'
import type { ThemeBlock } from '@/entities/theme/model/types'
import { cn } from '@/lib/utils'
import { FLUID_BLOCKS, type FluidBlockDefinition } from '@/features/fluid/components/fluidBlocks'

interface FluidBlocksPanelProps {
  blocks: ThemeBlock[]
  selectedIndex: number | null
  onSelectBlock: (index: number) => void
  onInsertBlock: (block: ThemeBlock, index: number) => void
  onRemoveBlock: (index: number) => void
  onReorderBlocks: (fromIndex: number, toIndex: number) => void
}

const BLOCK_LABELS: Map<string, string> = new Map(
  FLUID_BLOCKS.map((block) => [block.id, block.label]),
)


export function FluidBlocksPanel({
  blocks,
  selectedIndex,
  onSelectBlock,
  onInsertBlock,
  onRemoveBlock,
  onReorderBlocks,
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
    onInsertBlock(
      {
        block: definition.id,
        props: definition.props,
        children: [],
      },
      insertIndex,
    )
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
    onReorderBlocks(fromIndex, index)
  }

  const handleItemDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }

  const pickerOpen = openKey !== null

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
              onClick={() => handleOpenPicker('header', blocks.length)}
            >
              <Plus className="h-4 w-4" />
            </Button>,
          )}
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          <div className="space-y-3 text-xs">
            <TreeRow
              label="Page"
              depth={0}
              onAdd={() => handleOpenPicker('page', blocks.length)}
              renderButton={(button) => renderAnchorButton('page', button)}
            />
            <TreeRow
              label="Layout Container"
              depth={1}
              onAdd={() => handleOpenPicker('layout', blocks.length)}
              renderButton={(button) => renderAnchorButton('layout', button)}
            />
            <div className="space-y-1">
              {blocks.length === 0 && (
                <div className="text-muted-foreground pl-8 text-[11px]">
                  Add blocks to build this page.
                </div>
              )}
              {blocks.map((block, index) => {
                const key = `block-${index}`
                return (
                  <div
                    key={key}
                    className={cn(
                      'group flex items-center gap-2 rounded-md py-1 pl-8 pr-2 transition-colors',
                      selectedIndex === index
                        ? 'bg-primary/10 text-foreground'
                        : 'text-muted-foreground hover:bg-muted/40',
                    )}
                    draggable
                    onDragStart={(event) => handleItemDragStart(event, index)}
                    onDrop={(event) => handleItemDrop(event, index)}
                    onDragOver={handleItemDragOver}
                  >
                    <GripVertical className="h-3.5 w-3.5 text-muted-foreground/60" />
                    <button
                      type="button"
                      className="flex flex-1 items-center gap-2 text-left"
                      onClick={() => onSelectBlock(index)}
                    >
                      <span className="text-[11px] font-medium">
                        {BLOCK_LABELS.get(block.block) ?? block.block}
                      </span>
                    </button>
                    {renderAnchorButton(
                      key,
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
                        onClick={() => handleOpenPicker(key, index + 1)}
                      >
                        <Plus className="h-3.5 w-3.5" />
                      </Button>,
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
                      onClick={() => onRemoveBlock(index)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                )
              })}
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

function BlockPreview({ blockId }: { blockId: FluidBlockDefinition['id'] }) {
  switch (blockId) {
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
