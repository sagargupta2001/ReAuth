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
import {
  expandComponentNode,
  type ComponentThemeContext,
} from '@/features/fluid/lib/componentRegistry'
import { readableTextOn } from '@/lib/colorUtils'
import { alignItemsFor, justifyContentFor } from '@/features/fluid/lib/flexLayout'
import {
  getNestedRecord,
  resolveInputType,
  resolveThemeColor,
} from '@/features/fluid/lib/themeUtils'
import {
  computeNodeVisuals,
  resolveDisplayText,
  resolveRadius,
  resolveVisibleFlag,
} from '@/features/fluid/lib/nodeVisuals'
import type { ProviderPreview } from '@/features/fluid/model/providerPreview'

interface FluidCanvasProps {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  blocks: ThemeNode[]
  assets: ThemeAsset[]
  selectedNodeId: string | null
  /** Realm providers, so `ProviderButtons` previews what users will see. */
  providers?: ProviderPreview[]
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
  providers = [],
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

  const rawBackground = String(colors.background || '')
  const rawText = String(colors.text || '')
  const rawSurface = String(colors.surface || '')
  const rawPrimary = String(colors.primary || '')
  const radiusBase = Number.parseFloat(String(radius.base || '12')) || 12
  const shell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const assetMap = new Map(assets.map((asset) => [asset.id, asset]))
  const [hoveredIndex, setHoveredIndex] = useState<string | null>(null)

  const background = resolveThemeColor(rawBackground, 'var(--background)')
  const text = resolveThemeColor(rawText, 'var(--foreground)')
  const surface = resolveThemeColor(rawSurface, 'var(--card)')
  const primary = rawPrimary.trim() || 'var(--primary)'
  const componentTheme: ComponentThemeContext = { text, radius: radiusBase }
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
    if (!resolveVisibleFlag(node.props?.visible)) {
      return null
    }
    const {
      props,
      alignClass,
      sizeClass,
      widthClass,
      heightClass,
      fillWidthClass,
      fillHeightClass,
      style,
      fontSize,
      fontWeight,
      size,
      heightMode,
      heightValue,
    } = computeNodeVisuals(node)
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
        const alignItems = alignItemsFor(layout)
        const justifyContent = justifyContentFor(layout)
        const paddingValue = Array.isArray(layout.padding)
          ? layout.padding.map((value) => `${value}px`).join(' ')
          : undefined
        const borderColor = String(props.border_color || '')
        const borderWidth = Number.parseFloat(String(props.border_width || ''))
        const borderRadius = resolveRadius(props.radius)
        const background = String(props.background || '')
        const boxStyle: CSSProperties = {
          gap,
          alignItems,
          justifyContent,
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
      case 'Text': {
        const { text: displayText, isBinding } = resolveDisplayText(props)
        // The wrapper already carries font_size/font_weight/color from props, so
        // only apply the heading defaults when the node does not set them —
        // a utility class here would override the inherited inline style.
        return wrap(
          <div className={cn('py-1', alignClass)}>
            <p
              className={cn(
                !fontSize && 'text-lg',
                !fontWeight && 'font-semibold',
                isBinding && 'italic opacity-60',
              )}
              title={isBinding ? `Bound to context: ${String(props.text_path)}` : undefined}
            >
              {displayText}
            </p>
          </div>,
          undefined,
        )
      }
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
          // h-auto: the expanded field container supplies the height via its
          // padding, so the input must not add its own.
          'h-auto flex-1 border-0 bg-transparent px-0 py-0 shadow-none focus-visible:ring-0',
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
            <PasswordInput
              className="flex-1"
              inputClassName={inputClass}
              placeholder={placeholder}
              disabled
            />
          ) : (
            <Input className={inputClass} placeholder={placeholder} type={inputType} disabled />
          ),
          cn('flex-1', options?.wrapperClass),
        )
      }
      case 'Component': {
        const expanded = expandComponentNode(node, componentTheme)
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
            buttonStyle.backgroundColor = primary
            // Derived, not hard-coded: a white label on a light primary is invisible.
            buttonStyle.color = readableTextOn(primary)
          }
          if (variant === 'outline') {
            buttonStyle.borderColor = primary
            buttonStyle.color = primary
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
              className="text-xs underline underline-offset-2"
              style={{ color: fontColor || primary }}
              onClick={(event) => event.preventDefault()}
            >
              {label}
            </a>,
            alignClass,
          )
        }

        if (component.toLowerCase() === 'divider') {
          return wrap(<Separator />, cn('py-2'))
        }

        if (component.toLowerCase() === 'providerbuttons') {
          // The runtime hides this block when no providers are enabled. The
          // builder keeps a placeholder so the node stays visible and selectable.
          return wrap(
            providers.length === 0 ? (
              <div className="text-muted-foreground rounded-md border border-dashed px-3 py-4 text-center text-xs">
                No sign-in providers enabled for this realm.
              </div>
            ) : (
              <div className="flex w-full flex-col gap-3">
                {providers.map((provider) => {
                  const accent = provider.button_color || primary
                  return (
                    <Button
                      key={provider.alias}
                      type="button"
                      variant="outline"
                      className="w-full justify-center"
                      style={{ borderColor: accent, color: accent }}
                      disabled
                    >
                      {provider.display_name}
                    </Button>
                  )
                })}
              </div>
            ),
            options?.wrapperClass,
          )
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
    <section className="flex h-full flex-1 flex-col">
      {showChrome && (
        <div className="bg-background flex items-center justify-between  px-4 py-2">
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
                <form className="space-y-4" onSubmit={(event) => event.preventDefault()}>
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
              <form className="space-y-4" onSubmit={(event) => event.preventDefault()}>
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
