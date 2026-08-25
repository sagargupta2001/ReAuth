import type { CSSProperties, ReactNode } from 'react'

import { Separator } from '@/components/separator'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import { readableTextOn } from '@/lib/colorUtils'
import { cn } from '@/lib/utils'
import { renderIcon } from '@/shared/ui/icon-registry'
import {
  expandComponentNode,
  type ComponentThemeContext,
} from '@/features/fluid/lib/componentRegistry'
import { alignItemsFor, justifyContentFor } from '@/features/fluid/lib/flexLayout'
import {
  computeNodeVisuals,
  resolveRadius,
  type NodeVisuals,
} from '@/features/fluid/lib/nodeVisuals'
import { resolveInputType } from '@/features/fluid/lib/themeUtils'

/**
 * The one tree walker behind both Fluid renderers.
 *
 * `FluidCanvas` (builder preview) and `FluidLoginScreen` (runtime) used to hold
 * a node `switch` each — 312 and 352 lines, of which 233 were byte-identical.
 * That duplication is why `ProviderButtons` shipped working at runtime and
 * broken in the builder, why the two `resolveVisibleFlag` copies disagreed on
 * `visible: 0`, and why five renderer defaults had to be fixed twice.
 *
 * Structure and styling live here. A host supplies only the things that are
 * genuinely different between an inert, selectable preview and a live auth
 * page: how a node is wrapped, whether it renders at all, and what the
 * interactive leaves (text bindings, inputs, buttons, providers) actually are.
 *
 * Adding a block or a render-level styling prop is now one change, not two.
 */

export interface FluidRenderOptions {
  /** Extra classes for the node's wrapper, applied by every branch. */
  wrapperClass?: string
  /** Builder only: a Component's expansion must not be separately selectable. */
  disableSelection?: boolean
}

/** What the shared code has already computed for an `Input` leaf. */
export interface FluidInputSpec {
  name: string
  inputType: string
  placeholder: string
  /** Classes for the inner input element, not its wrapper. */
  inputClass: string
}

/** What the shared code has already computed for a `Button` leaf. */
export interface FluidButtonSpec {
  /** `props.label`, defaulted. The runtime may override it while busy. */
  defaultLabel: string
  /** Raw `props.variant`, for hosts that branch on `primary`. */
  variant: string
  buttonVariant: 'default' | 'secondary' | 'outline'
  className: string
  style: CSSProperties
}

export interface FluidWrapArgs {
  node: ThemeNode
  index: number
  content: ReactNode
  /** Branch-specific wrapper classes, already merged with `options`. */
  className?: string
  options?: FluidRenderOptions
  visuals: NodeVisuals
}

/**
 * The differences between the builder preview and the runtime, and nothing else.
 *
 * Anything added here should be a genuine behavioural difference. Styling and
 * structure belong in the shared walker, or the two will drift again.
 */
export interface FluidHost {
  /** Resolved primary colour, for button and link accents. */
  primary: string
  /** Theme context the component registry needs to expand an `Input`. */
  componentTheme: ComponentThemeContext
  assets: ReadonlyMap<string, ThemeAsset>
  /** Runtime gates on `visible` / `visible_if`; the builder always renders. */
  isVisible(node: ThemeNode): boolean
  /** The builder adds a selection ring and click target; the runtime does not. */
  wrap(args: FluidWrapArgs): ReactNode
  /** Resolved copy: the builder shows the binding, the runtime resolves it. */
  renderText(node: ThemeNode, visuals: NodeVisuals): ReactNode
  /** Inert preview input vs a form-wired one. */
  renderInput(node: ThemeNode, spec: FluidInputSpec): ReactNode
  /** Disabled button vs one carrying actions, OAuth, and passkeys. */
  renderButton(node: ThemeNode, spec: FluidButtonSpec): ReactNode
  /** Provider previews vs live buttons. `null` hides the block entirely. */
  renderProviders(node: ThemeNode): ReactNode | null
  /** Extra props for a `Link`'s anchor, e.g. the builder's `preventDefault`. */
  linkProps?(node: ThemeNode): { onClick?: (event: React.MouseEvent) => void }
}

export function renderFluidNode(
  node: ThemeNode,
  host: FluidHost,
  index = 0,
  options?: FluidRenderOptions,
): ReactNode {
  if (!host.isVisible(node)) {
    return null
  }

  const visuals = computeNodeVisuals(node)
  const {
    props,
    alignClass,
    sizeClass,
    fillHeightClass,
    innerWidthClass,
    innerHeightClass,
    size,
    heightMode,
    heightValue,
  } = visuals

  /**
   * Every branch goes through this, and it merges `options.wrapperClass` itself.
   * Branches used to merge it by hand and four of them forgot: brand-slot text
   * was white at runtime and not in the builder, and the Image, Link, and
   * Divider wrappers each differed between the two.
   */
  const wrap = (content: ReactNode, className?: string) =>
    host.wrap({
      node,
      index,
      content,
      className: cn(className, options?.wrapperClass),
      options,
      visuals,
    })

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
      // A bare number is not valid CSS ("border-radius: 12" is dropped), so
      // unitless values get px.
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
        <div
          className={cn('flex', direction, innerWidthClass, innerHeightClass)}
          style={boxStyle}
        >
          {(node.children ?? []).map((child, childIndex) =>
            renderFluidNode(child, host, childIndex, {
              disableSelection: options?.disableSelection,
            }),
          )}
        </div>,
      )
    }

    case 'Text':
      return wrap(<div className={cn('py-1', alignClass)}>{host.renderText(node, visuals)}</div>)

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
        'flex-0',
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
      return wrap(
        host.renderInput(node, { name, inputType, placeholder, inputClass }),
        'flex-1',
      )
    }

    case 'Component': {
      const expanded = expandComponentNode(node, host.componentTheme)
      if (expanded) {
        return wrap(
          renderFluidNode(expanded, host, index, { disableSelection: true }),
        )
      }
      const component = String(node.component || '').toLowerCase()
      const renderComponent = COMPONENT_RENDERERS[component]
      if (renderComponent) {
        return renderComponent({ node, host, visuals, wrap })
      }

      return wrap(
        <div className="text-xs text-muted-foreground">
          Unknown component: {String(node.component || '')}
        </div>,
      )
    }

    case 'Image': {
      const asset = host.assets.get(String(props.asset_id || ''))
      const height =
        heightMode === 'fixed' && heightValue
          ? heightValue
          : heightMode === 'fill'
            ? '100%'
            : heightValue || (size === 'sm' ? '120px' : size === 'lg' ? '240px' : '180px')
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


/** What a component renderer gets from the walker. */
export interface FluidComponentContext {
  node: ThemeNode
  host: FluidHost
  visuals: NodeVisuals
  /** Wraps content in the node's wrapper; the caller's class is already merged. */
  wrap: (content: ReactNode, className?: string) => ReactNode
}

export type FluidComponentRenderer = (context: FluidComponentContext) => ReactNode

/**
 * Renderers for `Component` nodes the registry does not expand, keyed by
 * lowercased component name.
 *
 * A map rather than an `if` chain so that adding a component block is an entry
 * here, and so the set is *enumerable*: the capability matrix reads these keys
 * directly instead of pattern-matching the source for branches.
 */
export const COMPONENT_RENDERERS: Record<string, FluidComponentRenderer> = {
  button: ({ node, host, visuals, wrap }) => {
    const { props, alignClass, sizeClass, fillWidthClass, fillHeightClass } = visuals
    const variant = String(props.variant || 'primary')
    const buttonVariant =
      variant === 'secondary' ? 'secondary' : variant === 'outline' ? 'outline' : 'default'
    const style: CSSProperties = {}
    if (variant === 'primary') {
      style.backgroundColor = host.primary
      // Derived, not hard-coded: a white label on a light primary is invisible.
      style.color = readableTextOn(host.primary)
    }
    if (variant === 'outline') {
      style.borderColor = host.primary
      style.color = host.primary
    }
    return wrap(
      host.renderButton(node, {
        defaultLabel: String(props.label || 'Continue'),
        variant,
        buttonVariant,
        className: cn(alignClass, sizeClass, fillWidthClass, fillHeightClass),
        style,
      }),
    )
  },

  link: ({ node, host, visuals, wrap }) => {
    const { props, alignClass } = visuals
    const target = String(props.target || '_self')
    const fontColor = String(props.color || '')
    return wrap(
      <a
        href={String(props.href || '#')}
        target={target}
        rel={target === '_blank' ? 'noreferrer' : undefined}
        className="text-xs underline underline-offset-2"
        style={{ color: fontColor || host.primary }}
        {...(host.linkProps?.(node) ?? {})}
      >
        {String(props.label || 'Link')}
      </a>,
      alignClass,
    )
  },

  divider: ({ wrap }) => wrap(<Separator />),

  providerbuttons: ({ node, host, wrap }) => {
    const providers = host.renderProviders(node)
    // The runtime hides the block outright when the realm has none; the builder
    // returns a placeholder so the node stays selectable.
    if (providers === null) return null
    return wrap(providers)
  },
}

/** Component names the walker renders inline, for coverage checks. */
export const RENDERED_COMPONENTS: readonly string[] = Object.keys(COMPONENT_RENDERERS)

/** Node types the walker has a branch for, for coverage checks. */
export const RENDERED_NODE_TYPES: readonly ThemeNode['type'][] = [
  'Box',
  'Text',
  'Icon',
  'Input',
  'Component',
  'Image',
]
