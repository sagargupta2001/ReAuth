import type { MouseEvent, ReactNode } from 'react'
import { useState } from 'react'
import type { CSSProperties } from 'react'
import { useForm } from 'react-hook-form'

import { Monitor, Smartphone, Tablet } from 'lucide-react'

import { Button } from '@/components/button'
import { Form } from '@/components/form'
import { Input } from '@/components/input'
import { Separator } from '@/components/separator'
import { Tabs, TabsList, TabsTrigger } from '@/components/tabs'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import { renderIcon } from '@/shared/ui/icon-registry'
import { PasswordInput } from '@/shared/ui/password-input'
import { cn } from '@/lib/utils'
import { expandComponentNode } from '@/features/fluid/lib/componentRegistry'
import {
  getNestedRecord,
  resolveInputType,
  resolveThemeColor,
  resolveThemeMode,
} from '@/features/fluid/lib/themeUtils'

interface FluidCanvasProps {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  blocks: ThemeNode[]
  assets: ThemeAsset[]
  selectedNodeId: string | null
  isInspecting?: boolean
  showChrome?: boolean
  onSelectNode: (nodeId: string) => void
}


export function FluidCanvas({
  tokens,
  layout,
  blocks,
  assets,
  selectedNodeId,
  isInspecting = false,
  showChrome = true,
  onSelectNode,
}: FluidCanvasProps) {
  const form = useForm<{ username: string; password: string }>({
    defaultValues: { username: '', password: '' },
  })
  const colors = getNestedRecord(tokens, 'colors')
  const typography = getNestedRecord(tokens, 'typography')
  const radius = getNestedRecord(tokens, 'radius')
  const appearance = getNestedRecord(tokens, 'appearance')

  const rawBackground = String(colors.background || '')
  const rawText = String(colors.text || '')
  const rawSurface = String(colors.surface || '')
  const rawPrimary = String(colors.primary || '')
  const radiusBase = Number.parseFloat(String(radius.base || '12')) || 12
  const shell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const assetMap = new Map(assets.map((asset) => [asset.id, asset]))
  const [hoveredIndex, setHoveredIndex] = useState<string | null>(null)
  const mode = String(appearance.mode || 'auto')
  const resolvedMode = resolveThemeMode(mode)
  const themeClass = resolvedMode === 'dark' ? 'dark' : resolvedMode === 'light' ? 'light' : ''

  const background = resolveThemeColor(
    rawBackground,
    resolvedMode,
    'var(--background)',
    ['#ffffff', '#fff', '#f8fafc'],
  )
  const text = resolveThemeColor(
    rawText,
    resolvedMode,
    'var(--foreground)',
    ['#0f172a', '#111827'],
  )
  const surface = resolveThemeColor(
    rawSurface,
    resolvedMode,
    'var(--card)',
    ['#ffffff', '#fff'],
  )
  const primary = rawPrimary.trim() || 'var(--primary)'
  const fontFamily = String(typography.font_family || 'system-ui')
  const baseSize = Number.parseFloat(String(typography.base_size || '16')) || 16
  const containerStyle: CSSProperties = {
    backgroundColor: background,
    color: text,
    fontFamily,
    fontSize: `${baseSize}px`,
  }

  const renderNode = (
    node: ThemeNode,
    options?: { wrapperClass?: string; disableSelection?: boolean },
  ): ReactNode => {
    const isVisible = (() => {
      const value = node.props?.visible
      if (value === undefined) return true
      if (typeof value === 'boolean') return value
      if (typeof value === 'string') return value.toLowerCase() !== 'false'
      return true
    })()
    if (!isVisible) {
      return null
    }
    const props = node.props ?? {}
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
    const widthMode = String(node.size?.width || props.width || 'fill')
    const widthValue = String(node.size?.width_value || props.width_value || '')
    const heightMode = String(node.size?.height || props.height || 'hug')
    const heightValue = String(node.size?.height_value || props.height_value || '')
    const size = String(props.size || 'md')
    const style: CSSProperties = {
      marginTop: `${marginTop}px`,
      marginBottom: `${marginBottom}px`,
      padding: `${padding}px`,
    }
    const widthClass =
      widthMode === 'hug' || widthMode === 'auto'
        ? 'w-auto'
        : widthMode === 'fixed' || widthMode === 'custom'
          ? ''
          : 'w-full'
    const heightClass =
      heightMode === 'fill'
        ? 'h-full'
        : heightMode === 'fixed'
          ? ''
          : 'h-auto'
    const fillHeightClass =
      heightMode === 'fill' || heightMode === 'fixed' ? 'h-full' : ''
    const fillWidthClass = widthMode === 'fill' ? 'w-full' : ''
    if ((widthMode === 'fixed' || widthMode === 'custom') && widthValue) {
      style.width = widthValue
    }
    if (heightMode === 'fixed' && heightValue) {
      style.height = heightValue
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
    const isSelected = selectedNodeId === node.id
    const isHoverable = isInspecting && !options?.disableSelection
    const handleSelect = (event: MouseEvent<HTMLDivElement>) => {
      if (!isInspecting || options?.disableSelection) return
      event.stopPropagation()
      onSelectNode(node.id)
    }
    const wrapperClass = cn(
      'transition-shadow',
      isInspecting ? 'cursor-pointer' : 'cursor-default',
      isSelected && 'ring-primary/40 ring-2 ring-offset-2 ring-offset-background rounded-md',
      isHoverable &&
        hoveredIndex === node.id &&
        'ring-primary/20 ring-2 ring-offset-2 ring-offset-background',
      options?.wrapperClass,
    )
    const sizeClassName = cn(widthClass, heightClass)
    const wrap = (content: ReactNode, className?: string) => (
      <div
        key={`node-${node.id}`}
        className={cn(wrapperClass, sizeClassName, className)}
        style={style}
        onClick={handleSelect}
        onMouseEnter={() => {
          if (isHoverable) {
            setHoveredIndex(node.id)
          }
        }}
        onMouseLeave={() => {
          if (isHoverable) {
            setHoveredIndex(null)
          }
        }}
      >
        {content}
      </div>
    )

    switch (node.type) {
      case 'Box': {
        const layout = node.layout ?? {}
        const direction = layout.direction === 'row' ? 'flex-row' : 'flex-col'
        const gap = typeof layout.gap === 'number' ? `${layout.gap}px` : undefined
        const alignItems =
          layout.align === 'center'
            ? 'center'
            : layout.align === 'end'
              ? 'flex-end'
              : layout.align === 'start'
                ? 'flex-start'
                : 'stretch'
        const paddingValue = Array.isArray(layout.padding)
          ? layout.padding.map((value) => `${value}px`).join(' ')
          : undefined
        const borderColor = String(props.border_color || '')
        const borderWidth = Number.parseFloat(String(props.border_width || ''))
        const borderRadius = String(props.radius || '')
        const background = String(props.background || '')
        const boxStyle: CSSProperties = {
          gap,
          alignItems,
          padding: paddingValue,
          backgroundColor: background || undefined,
          borderColor: borderColor || undefined,
          borderWidth: Number.isNaN(borderWidth) ? undefined : `${borderWidth}px`,
          borderStyle: borderColor || !Number.isNaN(borderWidth) ? 'solid' : undefined,
          borderRadius: borderRadius || undefined,
        }
        return wrap(
          <div className={cn('flex w-full', direction)} style={boxStyle}>
            {(node.children ?? []).map((child) =>
              renderNode(child, { disableSelection: options?.disableSelection }),
            )}
          </div>,
          undefined,
        )
      }
      case 'Text':
        return wrap(
          <div className={cn('py-1', alignClass)}>
            <p className="text-lg font-semibold">{String(props.text || 'Headline')}</p>
          </div>,
          undefined,
        )
      case 'Icon': {
        const name = String(props.name || '')
        const color = String(props.color || '')
        const sizeValue = Number.parseFloat(String(props.size || '16'))
        const svgPath = String(props.svg_path || '').trim()
        const svgViewBox = String(props.svg_viewbox || '').trim()
        return wrap(
          <span className="flex items-center justify-center">
            {renderIcon(
              name,
              { size: Number.isNaN(sizeValue) ? 16 : sizeValue, color: color || undefined },
              { svgPath, viewBox: svgViewBox || undefined },
            ) ?? (
              <span style={{ color: color || '#94a3b8', fontSize: `${sizeValue || 16}px` }}>
                {name ? name.charAt(0).toUpperCase() : '•'}
              </span>
            )}
          </span>,
          cn('flex-0', options?.wrapperClass),
        )
      }
      case 'Input': {
        const name = String(props.name || '')
        const inputType = resolveInputType(props, name || 'input')
        const placeholder = String(props.placeholder || '')
        const inputClass = cn(
          sizeClass,
          'flex-1 border-0 bg-transparent px-0 py-0 shadow-none focus-visible:ring-0',
          fillHeightClass,
        )
        if (isInspecting) {
          return wrap(
            <div
              className={cn(
                inputClass,
                'pointer-events-none flex items-center text-muted-foreground/70',
              )}
            >
              {placeholder || 'Input'}
            </div>,
            cn('flex-1', options?.wrapperClass),
          )
        }
        return wrap(
          inputType === 'password' ? (
            <PasswordInput className={inputClass} disabled />
          ) : (
            <Input className={inputClass} placeholder={placeholder} type={inputType} disabled />
          ),
          cn('flex-1', options?.wrapperClass),
        )
      }
      case 'Component': {
        const expanded = expandComponentNode(node)
        if (expanded) {
          return wrap(renderNode(expanded, { disableSelection: true }), options?.wrapperClass)
        }
        const component = String(node.component || '')

        if (component.toLowerCase() === 'button') {
          const variant = String(props.variant || 'primary')
          const buttonVariant =
            variant === 'secondary' ? 'secondary' : variant === 'outline' ? 'outline' : 'default'
          const buttonStyle: CSSProperties = {}
          if (variant === 'primary') {
            buttonStyle.backgroundColor = String(colors.primary || 'var(--primary)')
            buttonStyle.color = '#ffffff'
          }
          if (variant === 'outline') {
            buttonStyle.borderColor = String(colors.primary || 'var(--primary)')
            buttonStyle.color = String(colors.primary || 'var(--primary)')
          }
          return wrap(
            <Button
              type="button"
              variant={buttonVariant}
              className={cn(alignClass, sizeClass, fillWidthClass, fillHeightClass)}
              style={buttonStyle}
              disabled
            >
              {String(props.label || 'Continue')}
            </Button>,
            options?.wrapperClass,
          )
        }

        if (component.toLowerCase() === 'link') {
          const label = String(props.label || 'Link')
          const href = String(props.href || '#')
          const target = String(props.target || '_self')
          const isExternal = target === '_blank'
          const fontColor = String(props.color || '')
          return wrap(
            <a
              href={href}
              target={target}
              rel={isExternal ? 'noreferrer' : undefined}
              className={cn('text-xs underline', alignClass)}
              style={{ color: fontColor || primary }}
              onClick={(event) => event.preventDefault()}
            >
              {label}
            </a>,
            undefined,
          )
        }

        if (component.toLowerCase() === 'divider') {
          return wrap(<Separator />, cn('py-2'))
        }

        return wrap(
          <div className="text-xs text-muted-foreground">Unknown component: {component}</div>,
        )
      }
      case 'Image': {
        const assetId = String(props.asset_id || '')
        const asset = assetMap.get(assetId)
        const height =
          heightMode === 'fixed' && heightValue
            ? heightValue
            : heightMode === 'fill'
              ? '100%'
              : heightValue ||
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
          cn('flex flex-col', alignClass),
        )
      }
      default:
        return wrap(
          <div className="text-xs text-muted-foreground">Unknown node: {node.type}</div>,
        )
    }
  }

  const formBlocks = blocks.filter(
    (node) => !node.props || String(node.props.slot || 'form') === 'form',
  )
  const brandBlocks = blocks.filter(
    (node) => node.props && String(node.props.slot || '') === 'brand',
  )
  const nonSplitBlocks = blocks.filter(
    (node) => !node.props || String(node.props.slot || 'form') !== 'brand',
  )

  return (
    <section className={cn('flex h-full flex-1 flex-col', themeClass)}>
      {showChrome && (
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
        </div>
      )}

      <div
        className="relative flex flex-1 items-center justify-center overflow-auto p-8"
        style={containerStyle}
      >
        {shell === 'SplitScreen' ? (
          <div
            className="grid w-full max-w-4xl grid-cols-1 overflow-hidden rounded-2xl border shadow-lg md:grid-cols-2"
            style={{ backgroundColor: surface }}
          >
            <div className="flex flex-col justify-between bg-slate-900 p-8 text-white">
              {brandBlocks.length === 0 ? (
                <div className="space-y-2 text-xs text-white/60">
                  <div className="h-3 w-16 rounded-full bg-white/40" />
                  <div className="h-2 w-24 rounded-full bg-white/20" />
                  <p>Add brand blocks in Fluid.</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {brandBlocks.map((block) => (
                    <div key={`brand-${block.id}`} className="text-white">
                      {renderNode(block, { wrapperClass: 'text-white' })}
                    </div>
                  ))}
                </div>
              )}
              <div className="h-24 rounded-xl border border-white/10 bg-white/5" />
            </div>
            <div className="p-8" style={{ backgroundColor: background, color: text }}>
              <Form {...form}>
                <form className="space-y-3" onSubmit={(event) => event.preventDefault()}>
                  {formBlocks.length === 0 ? (
                    <div className="text-muted-foreground text-sm">
                      Add blocks to build this page.
                    </div>
                  ) : (
                    formBlocks.map((block) => renderNode(block))
                  )}
                </form>
              </Form>
            </div>
          </div>
        ) : (
          <div
            className={cn(
              'w-full max-w-md border p-8',
              shell === 'Minimal' ? 'border-transparent shadow-none' : 'shadow-lg',
            )}
            style={{
              borderRadius: `${radiusBase}px`,
              backgroundColor: shell === 'Minimal' ? 'transparent' : surface,
              color: text,
            }}
          >
            <Form {...form}>
              <form className="space-y-3" onSubmit={(event) => event.preventDefault()}>
                {nonSplitBlocks.length === 0 ? (
                  <div className="text-muted-foreground text-sm">
                    Add blocks to build this page.
                  </div>
                ) : (
                  nonSplitBlocks.map((block) => renderNode(block))
                )}
              </form>
            </Form>

            {showChrome && (
              <div className="mt-6 text-[10px] text-muted-foreground">
                Click a block to inspect it. Use the + buttons to add blocks.
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  )
}
