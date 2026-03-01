import type { MouseEvent, ReactNode } from 'react'
import { useState } from 'react'
import type { CSSProperties } from 'react'

import { Monitor, Smartphone, Tablet } from 'lucide-react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Separator } from '@/components/separator'
import { Tabs, TabsList, TabsTrigger } from '@/components/tabs'
import type { ThemeAsset, ThemeBlock } from '@/entities/theme/model/types'
import { cn } from '@/lib/utils'

interface FluidCanvasProps {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  blocks: ThemeBlock[]
  assets: ThemeAsset[]
  selectedIndex: number | null
  isInspecting?: boolean
  onSelectBlock: (index: number) => void
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

export function FluidCanvas({
  tokens,
  layout,
  blocks,
  assets,
  selectedIndex,
  isInspecting = false,
  onSelectBlock,
}: FluidCanvasProps) {
  const colors = getNestedRecord(tokens, 'colors')
  const radius = getNestedRecord(tokens, 'radius')

  const background = String(colors.background || '#F8FAFC')
  const text = String(colors.text || '#0F172A')
  const radiusBase = Number.parseFloat(String(radius.base || '12')) || 12
  const shell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const assetMap = new Map(assets.map((asset) => [asset.id, asset]))
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null)

  const renderBlock = (block: ThemeBlock, index: number, options?: { wrapperClass?: string }) => {
    const props = block.props ?? {}
    const align = String(props.align || 'left')
    const alignClass =
      align === 'center'
        ? 'text-center'
        : align === 'right'
          ? 'text-right'
          : 'text-left'
    const fontSize = String(props.font_size || '')
    const fontWeight = String(props.font_weight || '')
    const fontColor = String(props.color || '')
    const marginTop = Number.parseFloat(String(props.margin_top || '0')) || 0
    const marginBottom = Number.parseFloat(String(props.margin_bottom || '0')) || 0
    const padding = Number.parseFloat(String(props.padding || '0')) || 0
    const widthMode = String(props.width || 'full')
    const widthValue = String(props.width_value || '')
    const size = String(props.size || 'md')
    const style: CSSProperties = {
      marginTop: `${marginTop}px`,
      marginBottom: `${marginBottom}px`,
      padding: `${padding}px`,
    }
    const widthClass =
      widthMode === 'auto' ? 'w-auto' : widthMode === 'custom' ? '' : 'w-full'
    if (widthMode === 'custom' && widthValue) {
      style.width = widthValue
    }

    if (fontSize) {
      style.fontSize = fontSize
    }
    if (fontWeight) {
      const numeric = Number.parseInt(fontWeight, 10)
      style.fontWeight = Number.isNaN(numeric) ? fontWeight : numeric
    }
    if (fontColor) {
      style.color = fontColor
    }

    const sizeClass =
      size === 'sm' ? 'h-8 text-xs' : size === 'lg' ? 'h-11 text-base' : 'h-9 text-sm'
    const isSelected = selectedIndex === index
    const handleSelect = (event: MouseEvent<HTMLDivElement>) => {
      event.stopPropagation()
      if (!isInspecting) return
      onSelectBlock(index)
    }
    const wrapperClass = cn(
      'transition-shadow',
      isInspecting ? 'cursor-pointer' : 'cursor-default',
      isSelected && 'ring-primary/40 ring-2 ring-offset-2 ring-offset-background rounded-md',
      isInspecting &&
        hoveredIndex === index &&
        'ring-primary/20 ring-2 ring-offset-2 ring-offset-background',
      options?.wrapperClass,
    )
    const wrap = (content: ReactNode, className?: string) => (
      <div
        key={`block-${index}`}
        className={cn(wrapperClass, className)}
        style={style}
        onClick={handleSelect}
        onMouseEnter={() => {
          if (isInspecting) {
            setHoveredIndex(index)
          }
        }}
        onMouseLeave={() => {
          if (isInspecting) {
            setHoveredIndex(null)
          }
        }}
      >
        {content}
      </div>
    )

    switch (block.block) {
      case 'text':
        return wrap(
          <div className={cn('py-1', alignClass)}>
            <p className="text-lg font-semibold">{String(props.text || 'Headline')}</p>
          </div>,
          cn(widthClass, 'w-full'),
        )
      case 'input': {
        const name = String(props.name || '')
        const inputType = String(
          props.input_type || (name.toLowerCase().includes('password') ? 'password' : 'text'),
        )
        return wrap(
          <>
            <Label className="text-xs text-muted-foreground">
              {String(props.label || 'Field')}
            </Label>
            <Input
              placeholder={String(props.name || '')}
              type={inputType}
              className={cn('bg-white pointer-events-none', sizeClass)}
              disabled
            />
          </>,
          cn('space-y-1', alignClass, widthClass),
        )
      }
      case 'button': {
        const variant = String(props.variant || 'primary')
        const buttonVariant =
          variant === 'secondary' ? 'secondary' : variant === 'outline' ? 'outline' : 'default'
        return wrap(
          <Button
            variant={buttonVariant}
            className={cn(widthClass, alignClass, sizeClass, 'pointer-events-none')}
            disabled
          >
            {String(props.label || 'Continue')}
          </Button>,
          cn(alignClass, widthClass),
        )
      }
      case 'image': {
        const assetId = String(props.asset_id || '')
        const asset = assetMap.get(assetId)
        const heightValue = String(props.height_value || '')
        const height =
          heightValue ||
          (size === 'sm' ? '120px' : size === 'lg' ? '240px' : '180px')
        return wrap(
          asset ? (
            <img
              src={asset.url}
              alt={String(props.alt || asset.filename)}
              className="w-full rounded-lg object-cover"
              style={{ height }}
            />
          ) : (
            <div
              className="border-muted bg-muted/40 text-muted-foreground flex w-full items-center justify-center rounded-lg border text-xs"
              style={{ height }}
            >
              Select an asset
            </div>
          ),
          cn('flex flex-col', alignClass, widthClass),
        )
      }
      case 'divider':
        return wrap(<Separator />, cn('py-2'))
      default:
        return wrap(
          <div className="text-xs text-muted-foreground">Unknown block: {block.block}</div>,
        )
    }
  }

  const formBlocks = blocks.filter(
    (block) => !block.props || String(block.props.slot || 'form') === 'form',
  )
  const brandBlocks = blocks.filter(
    (block) => block.props && String(block.props.slot || '') === 'brand',
  )
  const nonSplitBlocks = blocks.filter(
    (block) => !block.props || String(block.props.slot || 'form') !== 'brand',
  )

  return (
    <section className="flex h-full flex-1 flex-col">
      <div className="bg-background flex items-center justify-between border-b px-4 py-2">
        <Tabs defaultValue="desktop" className="w-auto">
          <TabsList className="h-8">
            <TabsTrigger value="desktop" className="gap-2 text-xs">
              <Monitor className="h-3.5 w-3.5" /> Desktop
            </TabsTrigger>
            <TabsTrigger value="tablet" className="gap-2 text-xs">
              <Tablet className="h-3.5 w-3.5" /> Tablet
            </TabsTrigger>
            <TabsTrigger value="mobile" className="gap-2 text-xs">
              <Smartphone className="h-3.5 w-3.5" /> Mobile
            </TabsTrigger>
          </TabsList>
        </Tabs>

        <Button size="sm" variant="outline" disabled>
          Reset Layout
        </Button>
      </div>

      <div className="bg-muted/5 relative flex flex-1 items-center justify-center overflow-auto p-10">
        {isInspecting && (
          <div className="pointer-events-none absolute left-1/2 top-[72px] -translate-x-1/2 rounded-full border bg-background/90 px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground shadow-sm">
            Inspect Mode
          </div>
        )}
        {shell === 'SplitScreen' ? (
          <div className="grid w-full max-w-3xl grid-cols-2 overflow-hidden rounded-2xl border bg-white shadow-lg">
            <div className="flex flex-col justify-between bg-slate-900 p-6 text-white">
              {brandBlocks.length === 0 ? (
                <div className="space-y-2 text-xs text-white/60">
                  <div className="h-3 w-16 rounded-full bg-white/40" />
                  <div className="h-2 w-24 rounded-full bg-white/20" />
                  <p>Add brand blocks from the Sections panel.</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {brandBlocks.map((block, index) => (
                    <div key={`brand-${index}`} className="text-white">
                      {renderBlock(block, index, { wrapperClass: 'text-white' })}
                    </div>
                  ))}
                </div>
              )}
              <div className="h-32 rounded-xl border border-white/10 bg-white/5" />
            </div>
            <div
              className="p-8"
              style={{
                backgroundColor: background,
                color: text,
              }}
            >
              <div className="mb-6 space-y-2">
                <div className="h-4 w-24 rounded-full bg-slate-200" />
                <div className="h-2 w-32 rounded-full bg-slate-100" />
              </div>
              <div className="space-y-3">
                {formBlocks.length === 0 ? (
                  <div className="text-muted-foreground text-xs">
                    Add blocks with the + button to preview them here.
                  </div>
                ) : (
                  formBlocks.map((block, index) => renderBlock(block, index))
                )}
              </div>
            </div>
          </div>
        ) : (
          <div className="w-full max-w-md">
            <div
              className="border p-8 shadow-lg"
              style={{
                borderRadius: `${radiusBase}px`,
                backgroundColor: shell === 'Minimal' ? 'transparent' : background,
                color: text,
                boxShadow: shell === 'Minimal' ? 'none' : undefined,
                borderColor: shell === 'Minimal' ? 'transparent' : undefined,
              }}
            >
              <div className="mb-6 space-y-2">
                <div className="h-4 w-24 rounded-full bg-slate-200" />
                <div className="h-2 w-32 rounded-full bg-slate-100" />
              </div>
              <div className="space-y-3">
                {nonSplitBlocks.length === 0 ? (
                  <div className="text-muted-foreground text-xs">
                    Add blocks with the + button to preview them here.
                  </div>
                ) : (
                  nonSplitBlocks.map((block, index) => renderBlock(block, index))
                )}
              </div>
              <div className="mt-6 flex items-center justify-between text-[10px] text-slate-400">
                <span>Need help?</span>
                <span>Privacy Policy</span>
              </div>
            </div>

            <div className="mt-6 text-[10px] text-muted-foreground">
              Click a block to inspect it. Use the + buttons to add blocks.
            </div>
          </div>
        )}
      </div>
    </section>
  )
}
