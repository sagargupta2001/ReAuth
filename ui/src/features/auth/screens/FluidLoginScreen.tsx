import { useEffect, useMemo, useState } from 'react'
import type { CSSProperties, ReactNode } from 'react'

import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/form'
import { FormInput } from '@/shared/ui/form-input'
import { PasswordInput } from '@/shared/ui/password-input'
import { loginSchema } from '@/features/auth/schema/login.schema'
import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'
import type { ThemeBlock } from '@/entities/theme/model/types'
import { useThemeSnapshot } from '@/features/theme/api/useThemeSnapshot'
import { cn } from '@/lib/utils'
import { UsernamePasswordScreen } from '@/features/auth/screens/UsernamePasswordScreen'

type LoginFormValues = Record<string, string>

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

export function FluidLoginScreen({
  onSubmit,
  isLoading,
  error,
  context,
  realm = 'master',
  clientId,
}: AuthScreenProps) {
  const templateKey =
    typeof context?.template_key === 'string' ? (context.template_key as string) : 'login'

  const { data: snapshot, isLoading: isThemeLoading } = useThemeSnapshot(realm, {
    pageKey: templateKey,
    clientId,
  })
  const [localError, setLocalError] = useState<string | null>(null)

  const form = useForm<LoginFormValues>({
    defaultValues: {
      username: (context?.username as string) || '',
      password: '',
    },
  })

  useEffect(() => {
    if (context?.username) {
      form.setValue('username', context.username as string)
    }
  }, [context?.username, form])

  const displayError = localError || error || (context?.error as string) || null

  const tokens = useMemo(() => snapshot?.tokens ?? {}, [snapshot])
  const layout = useMemo(() => snapshot?.layout ?? { shell: 'CenteredCard' }, [snapshot])
  const blocks = useMemo(() => snapshot?.blocks ?? [], [snapshot])
  const assets = useMemo(() => snapshot?.assets ?? [], [snapshot])

  const colors = getNestedRecord(tokens, 'colors')
  const typography = getNestedRecord(tokens, 'typography')
  const radius = getNestedRecord(tokens, 'radius')
  const appearance = getNestedRecord(tokens, 'appearance')

  const rawBackground = String(colors.background || '')
  const rawText = String(colors.text || '')
  const rawPrimary = String(colors.primary || '')
  const rawSurface = String(colors.surface || '')
  const radiusBase = Number.parseFloat(String(radius.base || '12')) || 12
  const fontFamily = String(typography.font_family || 'system-ui')
  const baseSize = Number.parseFloat(String(typography.base_size || '16')) || 16
  const shell = typeof layout.shell === 'string' ? layout.shell : 'CenteredCard'
  const assetMap = new Map(assets.map((asset) => [asset.id, asset]))
  const themeMode = String(appearance.mode || 'auto')
  const resolvedMode = resolveThemeMode(themeMode)
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
  const primary = rawPrimary.trim() || 'var(--primary)'
  const surface = resolveThemeColor(
    rawSurface,
    resolvedMode,
    'var(--card)',
    ['#ffffff', '#fff'],
  )

  const formBlocks = useMemo(
    () =>
      blocks.filter(
        (block) => !block.props || String(block.props.slot || 'form') === 'form',
      ),
    [blocks],
  )
  const brandBlocks = useMemo(
    () =>
      blocks.filter(
        (block) => block.props && String(block.props.slot || '') === 'brand',
      ),
    [blocks],
  )
  const nonSplitBlocks = useMemo(
    () =>
      blocks.filter(
        (block) => !block.props || String(block.props.slot || 'form') !== 'brand',
      ),
    [blocks],
  )

  const handleSubmit = form.handleSubmit((values) => {
    setLocalError(null)
    const normalized = { ...values }
    if (!normalized.username && normalized.email) {
      normalized.username = normalized.email
    }
    const parsed = loginSchema.safeParse(normalized)
    if (!parsed.success) {
      setLocalError(parsed.error.issues[0]?.message ?? 'Invalid login details.')
      return
    }
    void onSubmit(normalized)
  })

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

    const wrap = (content: ReactNode, className?: string) => (
      <div
        key={`block-${index}`}
        className={cn(widthClass, className)}
        style={style}
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
          options?.wrapperClass,
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
              name={name}
              render={({ field }) => (
                <FormItem className={cn('space-y-1', alignClass)}>
                  <FormLabel className="text-xs text-muted-foreground">
                    {String(props.label || 'Field')}
                  </FormLabel>
                  <FormControl>
                    <PasswordInput {...field} className={cn(sizeClass)} disabled={isLoading} />
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
            name={name}
            label={String(props.label || 'Field')}
            placeholder={String(props.placeholder || '')}
            type={inputType}
            className={cn(sizeClass)}
            disabled={isLoading}
          />,
          options?.wrapperClass,
        )
      }
      case 'button': {
        const variant = String(props.variant || 'primary')
        const buttonVariant =
          variant === 'secondary' ? 'secondary' : variant === 'outline' ? 'outline' : 'default'
        const buttonStyle: React.CSSProperties = {}
        if (variant === 'primary') {
          buttonStyle.backgroundColor = primary
          buttonStyle.color = '#ffffff'
        }
        if (variant === 'outline') {
          buttonStyle.borderColor = primary
          buttonStyle.color = primary
        }
        return wrap(
          <Button
            type="submit"
            variant={buttonVariant}
            className={cn(widthClass, alignClass, sizeClass)}
            style={buttonStyle}
            disabled={isLoading}
          >
            {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
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
          options?.wrapperClass,
        )
      }
      case 'divider':
        return wrap(<Separator />, options?.wrapperClass)
      case 'link': {
        const label = String(props.label || 'Link')
        const href = String(props.href || '#')
        const target = String(props.target || '_self')
        const isExternal = target === '_blank'
        return wrap(
          <a
            href={href}
            target={target}
            rel={isExternal ? 'noreferrer' : undefined}
            className={cn('text-xs underline', alignClass)}
            style={{ color: fontColor || primary }}
          >
            {label}
          </a>,
          options?.wrapperClass,
        )
      }
      default:
        return wrap(
          <div className="text-xs text-muted-foreground">Unknown block: {block.block}</div>,
        )
    }
  }

  if (isThemeLoading && !snapshot) {
    return (
      <div className="flex justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
      </div>
    )
  }

  if (!snapshot) {
    return (
      <UsernamePasswordScreen
        onSubmit={onSubmit}
        isLoading={isLoading}
        error={error}
        context={context}
        realm={realm}
        clientId={clientId}
      />
    )
  }

  const containerStyle: React.CSSProperties = {
    backgroundColor: background,
    color: text,
    fontFamily,
    fontSize: `${baseSize}px`,
  }

  return (
    <div className={cn('min-h-svh w-full', themeClass)} style={containerStyle}>
      <div className="flex min-h-svh w-full items-center justify-center p-8">
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
                <form onSubmit={handleSubmit} className="space-y-3">
                  {displayError && (
                    <div className="text-destructive mb-2 rounded-md bg-red-50 p-3 text-sm font-medium">
                      {String(displayError)}
                    </div>
                  )}
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
              <form onSubmit={handleSubmit} className="space-y-3">
                {displayError && (
                  <div className="text-destructive mb-2 rounded-md bg-red-50 p-3 text-sm font-medium">
                    {String(displayError)}
                  </div>
                )}
                {nonSplitBlocks.length === 0 ? (
                  <div className="text-muted-foreground text-sm">
                    Add blocks to build this page.
                  </div>
                ) : (
                  nonSplitBlocks.map((block, index) => renderBlock(block, index))
                )}
              </form>
            </Form>
          </div>
        )}
      </div>
    </div>
  )
}
