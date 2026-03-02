import type { MouseEvent, ReactNode } from 'react'
import { useState } from 'react'
import type { CSSProperties } from 'react'
import { useForm } from 'react-hook-form'

import { Monitor, Smartphone, Tablet } from 'lucide-react'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { Separator } from '@/components/separator'
import { Tabs, TabsList, TabsTrigger } from '@/components/tabs'
import type { ThemeAsset, ThemeBlock } from '@/entities/theme/model/types'
import { FormInput } from '@/shared/ui/form-input'
import { PasswordInput } from '@/shared/ui/password-input'
import { cn } from '@/lib/utils'

interface FluidCanvasProps {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  blocks: ThemeBlock[]
  assets: ThemeAsset[]
  selectedIndex: number | null
  isInspecting?: boolean
  showChrome?: boolean
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

function resolveThemeColor(value: string, mode: string, fallback: string, legacy: string[]) {
  const trimmed = value.trim()
  if (!trimmed) return fallback
  const hslVarMatch = trimmed.match(/^hsl\(\s*(var\(--[^)]+\))\s*\)$/i)
  if (hslVarMatch) {
    return hslVarMatch[1]
  }
  const normalized = trimmed.toLowerCase()
  if (mode === 'dark' && legacy.includes(normalized)) {
    return fallback
  }
  return trimmed
}

function resolveThemeMode(mode: string) {
  if (mode !== 'auto') return mode
  if (typeof window === 'undefined') return 'light'
  if (document?.documentElement?.classList?.contains('dark')) return 'dark'
  if (document?.documentElement?.classList?.contains('light')) return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function resolveInputType(props: Record<string, unknown>, name: string) {
  const explicit = String(props.input_type || '').trim()
  if (explicit) return explicit
  if (name.toLowerCase().includes('password')) return 'password'
  return 'text'
}

export function FluidCanvas({
  tokens,
  layout,
  blocks,
  assets,
  selectedIndex,
  isInspecting = false,
  showChrome = true,
  onSelectBlock,
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
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null)
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
        if (!name) {
          return wrap(
            <div className="text-xs text-muted-foreground">Missing field name.</div>,
            options?.wrapperClass,
          )
        }
        const inputType = resolveInputType(props, name)
        if (inputType === 'password') {
          return wrap(
            <FormField
              control={form.control}
              name={name as 'password'}
              render={({ field }) => (
                <FormItem className={cn('space-y-1', alignClass)}>
                  <FormLabel className="text-xs text-muted-foreground">
                    {String(props.label || 'Field')}
                  </FormLabel>
                  <FormControl>
                    <PasswordInput {...field} className={cn(sizeClass)} disabled />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />,
            options?.wrapperClass,
          )
        }

        return wrap(
          <FormInput
            control={form.control}
            name={name as 'username'}
            label={String(props.label || 'Field')}
            placeholder={String(props.placeholder || '')}
            type={inputType}
            className={cn(sizeClass)}
            disabled
          />,
          options?.wrapperClass,
        )
      }
      case 'button': {
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
            className={cn(widthClass, alignClass, sizeClass)}
            style={buttonStyle}
            disabled
          >
            {String(props.label || 'Continue')}
          </Button>,
          options?.wrapperClass,
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
      case 'link': {
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
          cn(widthClass),
        )
      }
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

          <Button size="sm" variant="outline" disabled>
            Reset Layout
          </Button>
        </div>
      )}

      <div
        className="relative flex flex-1 items-center justify-center overflow-auto p-8"
        style={containerStyle}
      >
        {isInspecting && (
          <div className="pointer-events-none absolute left-1/2 top-[72px] -translate-x-1/2 rounded-full border bg-background/90 px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground shadow-sm">
            Inspect Mode
          </div>
        )}
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
                  {brandBlocks.map((block, index) => (
                    <div key={`brand-${index}`} className="text-white">
                      {renderBlock(block, index, { wrapperClass: 'text-white' })}
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
                    formBlocks.map((block, index) => renderBlock(block, index))
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
                  nonSplitBlocks.map((block, index) => renderBlock(block, index))
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
