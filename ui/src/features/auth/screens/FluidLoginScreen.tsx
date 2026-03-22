import { useEffect, useMemo, useState } from 'react'
import type { CSSProperties, ReactNode } from 'react'

import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { Form, FormField } from '@/components/form'
import { Input } from '@/components/input'
import { PasswordInput } from '@/shared/ui/password-input'
import { loginSchema } from '@/features/auth/schema/login.schema'
import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'
import type { ThemeNode } from '@/entities/theme/model/types'
import { useThemeSnapshot } from '@/features/theme/api/useThemeSnapshot'
import { cn } from '@/lib/utils'
import { UsernamePasswordScreen } from '@/features/auth/screens/UsernamePasswordScreen'
import { renderIcon } from '@/shared/ui/icon-registry'
import { expandComponentNode } from '@/features/fluid/lib/componentRegistry'
import {
  getNestedRecord,
  resolveInputType,
  resolveThemeColor,
  resolveThemeMode,
} from '@/features/fluid/lib/themeUtils'

type LoginFormValues = Record<string, string>

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
      otp: '',
    },
  })

  useEffect(() => {
    if (context?.username) {
      form.setValue('username', context.username as string)
    }
  }, [context?.username, form])

  useEffect(() => {
    if (templateKey === 'forgot_credentials' && context?.email) {
      form.setValue('email', context.email as string)
    }
  }, [context?.email, form, templateKey])

  const displayError = localError || error || (context?.error as string) || null

  const tokens = useMemo(() => snapshot?.tokens ?? {}, [snapshot])
  const layout = useMemo(() => snapshot?.layout ?? { shell: 'CenteredCard' }, [snapshot])
  const nodes = useMemo<ThemeNode[]>(() => snapshot?.nodes ?? [], [snapshot])
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
      nodes.filter(
        (node) => !node.props || String(node.props.slot || 'form') === 'form',
      ),
    [nodes],
  )
  const brandBlocks = useMemo(
    () =>
      nodes.filter(
        (node) => node.props && String(node.props.slot || '') === 'brand',
      ),
    [nodes],
  )
  const nonSplitBlocks = useMemo(
    () =>
      nodes.filter(
        (node) => !node.props || String(node.props.slot || 'form') !== 'brand',
      ),
    [nodes],
  )

  const handleSubmit = form.handleSubmit((values) => {
    setLocalError(null)
    const normalized = { ...values }
    if (!normalized.username && normalized.email) {
      normalized.username = normalized.email
    }
    if (templateKey === 'forgot_credentials') {
      if (!normalized.username) {
        setLocalError('Email or username is required.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'mfa') {
      const otp =
        normalized.otp || normalized.code || normalized.token || normalized.verification_code
      if (!otp) {
        setLocalError('Verification code is required.')
        return
      }
      void onSubmit({ otp })
      return
    }
    if (templateKey === 'verify_email') {
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'reset_password') {
      const minLength =
        typeof context?.min_password_length === 'number'
          ? context.min_password_length
          : 8
      if (!normalized.password) {
        setLocalError('Password is required.')
        return
      }
      if (String(normalized.password).length < minLength) {
        setLocalError(`Password must be at least ${minLength} characters.`)
        return
      }
      const confirm =
        normalized.password_confirm ||
        normalized.confirm_password ||
        normalized.password_confirmation
      if (confirm && confirm !== normalized.password) {
        setLocalError('Passwords do not match.')
        return
      }
      void onSubmit(normalized)
      return
    }
    if (templateKey === 'consent') {
      if (!normalized.decision) {
        setLocalError('Select allow or deny to continue.')
        return
      }
      void onSubmit(normalized)
      return
    }
    const parsed = loginSchema.safeParse(normalized)
    if (!parsed.success) {
      setLocalError(parsed.error.issues[0]?.message ?? 'Invalid login details.')
      return
    }
    void onSubmit(normalized)
  })

  const resolveContextValue = (path: string): unknown => {
    const trimmed = path.trim()
    if (!trimmed) return undefined
    const parts = trimmed.split('.')
    let current: unknown = context
    for (const part of parts) {
      if (!part) continue
      if (!current || typeof current !== 'object') return undefined
      current = (current as Record<string, unknown>)[part]
    }
    return current
  }

  const coerceVisible = (value: unknown): boolean => {
    if (value === undefined || value === null) return false
    if (typeof value === 'boolean') return value
    if (typeof value === 'number') return value !== 0
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase()
      if (!normalized || normalized === 'false' || normalized === '0') return false
      if (normalized === 'true') return true
      return true
    }
    return Boolean(value)
  }

  const resolveVisibleFlag = (value: unknown): boolean => {
    if (value === undefined) return true
    if (typeof value === 'boolean') return value
    if (typeof value === 'string') return value.toLowerCase() !== 'false'
    return Boolean(value)
  }

  const resolveVisibleIf = (value: unknown): boolean => {
    if (value === undefined) return true
    if (typeof value === 'boolean') return value
    if (typeof value === 'string') {
      const trimmed = value.trim()
      if (!trimmed) return true
      const lowered = trimmed.toLowerCase()
      if (lowered === 'true') return true
      if (lowered === 'false') return false
      return coerceVisible(resolveContextValue(trimmed))
    }
    return Boolean(value)
  }

  const renderNode = (
    node: ThemeNode,
    index: number,
    options?: { wrapperClass?: string },
  ): ReactNode => {
    const props = node.props ?? {}
    const isVisible =
      resolveVisibleFlag(props.visible) && resolveVisibleIf(props.visible_if)
    if (!isVisible) {
      return null
    }
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

    const sizeClassName = cn(widthClass, heightClass)
    const wrap = (content: ReactNode, className?: string) => (
      <div
        key={`block-${index}`}
        className={cn(sizeClassName, className)}
        style={style}
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
            {(node.children ?? []).map((child, childIndex) =>
              renderNode(child, childIndex),
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
          options?.wrapperClass,
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
        return wrap(
          name ? (
            <FormField
              control={form.control}
              name={name}
              render={({ field }) =>
                inputType === 'password' ? (
                  <PasswordInput
                    {...field}
                    className={inputClass}
                    placeholder={placeholder}
                    disabled={isLoading}
                  />
                ) : (
                  <Input
                    {...field}
                    className={inputClass}
                    placeholder={placeholder}
                    type={inputType}
                    disabled={isLoading}
                  />
                )
              }
            />
          ) : inputType === 'password' ? (
            <PasswordInput className={inputClass} disabled={isLoading} />
          ) : (
            <Input
              className={inputClass}
              placeholder={placeholder}
              type={inputType}
              disabled={isLoading}
            />
          ),
          cn('flex-1', options?.wrapperClass),
        )
      }
      case 'Component': {
        const expanded = expandComponentNode(node)
        if (expanded) {
          return wrap(renderNode(expanded, index), options?.wrapperClass)
        }
        const component = String(node.component || '')

        if (component.toLowerCase() === 'button') {
          const variant = String(props.variant || 'primary')
          const intent = typeof props.intent === 'string' ? props.intent.trim() : ''
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
              className={cn(alignClass, sizeClass, fillWidthClass, fillHeightClass)}
              style={buttonStyle}
              disabled={isLoading}
              onClick={() => {
                if (intent) {
                  form.setValue('decision', intent)
                }
              }}
            >
              {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
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

        if (component.toLowerCase() === 'divider') {
          return wrap(<Separator />, options?.wrapperClass)
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
          options?.wrapperClass,
        )
      }
      default:
        return wrap(
          <div className="text-xs text-muted-foreground">Unknown node: {node.type}</div>,
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
                      {renderNode(block, index, { wrapperClass: 'text-white' })}
                    </div>
                  ))}
                </div>
              )}
              <div className="h-24 rounded-xl border border-white/10 bg-white/5" />
            </div>
            <div className="p-8" style={{ backgroundColor: background, color: text }}>
              <Form {...form}>
                <form onSubmit={handleSubmit} className="space-y-3">
                  {templateKey === 'consent' ? (
                    <input type="hidden" {...form.register('decision')} />
                  ) : null}
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
                    formBlocks.map((block, index) => renderNode(block, index))
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
                {templateKey === 'consent' ? (
                  <input type="hidden" {...form.register('decision')} />
                ) : null}
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
                  nonSplitBlocks.map((block, index) => renderNode(block, index))
                )}
              </form>
            </Form>
          </div>
        )}
      </div>
    </div>
  )
}
