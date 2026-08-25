import type { DragEvent, MouseEvent, ReactNode } from 'react'
import { useMemo, useState } from 'react'
import type { CSSProperties } from 'react'
import { useForm } from 'react-hook-form'

import { Monitor, Smartphone, Tablet } from 'lucide-react'

import { Button } from '@/components/button'
import { Form } from '@/components/form'
import { Checkbox } from '@/components/checkbox'
import { Input } from '@/components/input'
import { Tabs, TabsList, TabsTrigger } from '@/components/tabs'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import { PasswordInput } from '@/shared/ui/password-input'
import { cn } from '@/lib/utils'
import type { ComponentThemeContext } from '@/features/fluid/lib/componentRegistry'
import { getNestedRecord, resolveThemeColor } from '@/features/fluid/lib/themeUtils'
import { resolveDisplayText } from '@/features/fluid/lib/nodeVisuals'
import {
  renderFluidNode,
  type FluidHost,
  type FluidRenderOptions,
} from '@/features/fluid/lib/renderFluidNode'
import { FluidShell } from '@/features/fluid/components/FluidShell'
import { partitionShellBlocks } from '@/features/fluid/lib/shellBlocks'
import type { FluidDragController } from '@/features/fluid/hooks/useFluidDrag'
import { canAcceptChildren } from '@/features/fluid/model/blockCatalog'
import {
  DropAxis,
  axisForDirection,
  dropIntentForPoint,
} from '@/features/fluid/model/dropZones'
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
  /**
   * The builder's shared drag session. Absent in read-only previews, which is
   * what keeps the canvas inert outside the builder.
   */
  drag?: FluidDragController
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
  showChrome = false,
  drag,
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
  const [hoveredIndex, setHoveredIndex] = useState<string | null>(null)
  const assetMap = useMemo(
    () => new Map(assets.map((asset) => [asset.id, asset])),
    [assets],
  )

  const background = resolveThemeColor(rawBackground, 'var(--background)')
  const text = resolveThemeColor(rawText, 'var(--foreground)')
  const surface = resolveThemeColor(rawSurface, 'var(--card)')
  const primary = rawPrimary.trim() || 'var(--primary)'
  // Memoised because the render host depends on it; a fresh object each render
  // would rebuild the host, and with it every node, on every keystroke.
  const componentTheme: ComponentThemeContext = useMemo(
    () => ({ text, radius: radiusBase }),
    [text, radiusBase],
  )
  const fontFamily = String(typography.font_family || 'system-ui')
  const baseSize = Number.parseFloat(String(typography.base_size || '16')) || 16
  const containerStyle: CSSProperties = {
    backgroundColor: background,
    color: text,
    fontFamily,
    fontSize: `${baseSize}px`,
  }

  /**
   * The builder half of the shared renderer: selectable, inert, and always
   * visible so a hidden node can still be edited. Everything structural lives
   * in `renderFluidNode`.
   */
  const host: FluidHost = useMemo(
    () => ({
      primary,
      componentTheme,
      assets: assetMap,
      // The builder deliberately ignores `visible` / `visible_if`: a node the
      // runtime would hide must stay selectable here.
      isVisible: () => true,
      wrap: ({ node, content, className, options, visuals }) => {
        const isSelected = selectedNodeId === node.id
        // A Component's expansion parts are render-time only. They are never
        // selectable and never participate in a drag — the authored node is.
        const isAuthored = !options?.disableSelection
        const isHoverable = isInspecting && isAuthored
        const isDraggable = Boolean(drag) && isInspecting && isAuthored
        const acceptsChildren = canAcceptChildren(node)
        const axis = axisForDirection(options?.parentDirection)
        const dropState = drag?.dropTarget?.nodeId === node.id ? drag.dropTarget : null
        const isDropInside = dropState?.intent === 'inside'

        const intentAt = (event: DragEvent<HTMLDivElement>) =>
          dropIntentForPoint(event, event.currentTarget.getBoundingClientRect(), {
            acceptsChildren,
            axis,
          })

        return (
          <div
            key={`node-${node.id}`}
            className={cn(
              'relative transition-shadow',
              isInspecting ? 'cursor-pointer' : 'cursor-default',
              isSelected &&
                'ring-primary/40 ring-2 ring-offset-2 ring-offset-background rounded-md',
              isHoverable &&
                hoveredIndex === node.id &&
                'ring-primary/20 ring-2 ring-offset-2 ring-offset-background',
              drag?.draggingNodeId === node.id && 'opacity-50',
              isDropInside &&
                (dropState.isAllowed
                  ? 'ring-primary bg-primary/5 rounded-md ring-2'
                  : 'ring-destructive/60 rounded-md ring-2'),
              visuals.widthClass,
              visuals.heightClass,
              className,
            )}
            style={visuals.style}
            draggable={isDraggable}
            data-drop-intent={dropState?.intent}
            data-drop-allowed={dropState ? String(dropState.isAllowed) : undefined}
            onDragStart={
              isDraggable
                ? (event) => {
                    // Selecting what you picked up keeps the inspector on it.
                    onSelectNode(node.id)
                    drag?.onDragStart(event, node.id)
                  }
                : undefined
            }
            onDragEnd={isDraggable ? drag?.onDragEnd : undefined}
            onDragOver={
              isDraggable
                ? (event) => drag?.onDragOverIntent(event, node.id, intentAt(event))
                : undefined
            }
            onDragLeave={isDraggable ? (event) => drag?.onDragLeave(event, node.id) : undefined}
            onDrop={
              isDraggable
                ? (event) => drag?.onDropIntent(event, node.id, intentAt(event))
                : undefined
            }
            onClick={(event: MouseEvent<HTMLDivElement>) => {
              if (!isInspecting || !isAuthored) return
              event.stopPropagation()
              onSelectNode(node.id)
            }}
            onMouseEnter={() => {
              if (isHoverable) setHoveredIndex(node.id)
            }}
            onMouseLeave={() => {
              if (isHoverable) setHoveredIndex(null)
            }}
          >
            {dropState && !isDropInside && (
              <span
                aria-hidden="true"
                // Absolutely positioned so showing it never nudges the thing
                // being aimed at, and oriented along the parent's axis.
                className={cn(
                  'pointer-events-none absolute z-10 rounded-full',
                  dropState.isAllowed ? 'bg-primary' : 'bg-destructive',
                  axis === DropAxis.Horizontal
                    ? cn('inset-y-0 w-0.5', dropState.intent === 'before' ? 'left-0' : 'right-0')
                    : cn('inset-x-0 h-0.5', dropState.intent === 'before' ? 'top-0' : 'bottom-0'),
                )}
              />
            )}
            {content}
          </div>
        )
      },
      renderText: (_node, visuals) => {
        // No auth context here, so a bound node shows the binding itself.
        const { text: displayText, isBinding } = resolveDisplayText(visuals.props)
        return (
          <p
            className={cn(
              !visuals.fontSize && 'text-lg',
              !visuals.fontWeight && 'font-semibold',
              isBinding && 'italic opacity-60',
            )}
            title={isBinding ? `Bound to context: ${String(visuals.props.text_path)}` : undefined}
          >
            {displayText}
          </p>
        )
      },
      renderInput: (_node, { inputType, placeholder, inputClass }) => {
        if (isInspecting) {
          return (
            <div
              className={cn(
                inputClass,
                'pointer-events-none flex items-center text-muted-foreground/70',
              )}
            >
              {placeholder || 'Input'}
            </div>
          )
        }
        return inputType === 'password' ? (
          <PasswordInput
            className="flex-1"
            inputClassName={inputClass}
            placeholder={placeholder}
            disabled
          />
        ) : (
          <Input className={inputClass} placeholder={placeholder} type={inputType} disabled />
        )
      },
      renderButton: (_node, { defaultLabel, buttonVariant, className, style }) => (
        <Button type="button" variant={buttonVariant} className={className} style={style} disabled>
          {defaultLabel}
        </Button>
      ),
      renderCheckbox: (_node, { label, defaultChecked, controlId }) => (
        <div className="flex items-center gap-2">
          <Checkbox id={controlId} checked={defaultChecked} disabled />
          {label && (
            <label htmlFor={controlId} className="text-sm">
              {label}
            </label>
          )}
        </div>
      ),
      renderProviders: () =>
        providers.length === 0 ? (
          // The runtime hides the block when no providers are enabled. The
          // builder keeps a placeholder so the node stays visible and selectable.
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
      linkProps: () => ({ onClick: (event) => event.preventDefault() }),
    }),
    [
      primary,
      componentTheme,
      assetMap,
      selectedNodeId,
      isInspecting,
      hoveredIndex,
      onSelectNode,
      providers,
      drag,
    ],
  )

  const renderNode = (node: ThemeNode, options?: FluidRenderOptions): ReactNode =>
    renderFluidNode(node, host, 0, options)

  const { brand: brandBlocks, form: formBlocks, nonSplit: nonSplitBlocks } =
    partitionShellBlocks(blocks)

  const previewForm = (paneBlocks: ThemeNode[]) => (
    <Form {...form}>
      <form className="space-y-4" onSubmit={(event) => event.preventDefault()}>
        {paneBlocks.length === 0 ? (
          <div className="text-muted-foreground text-sm">Add blocks to build this page.</div>
        ) : (
          paneBlocks.map((block) => renderNode(block))
        )}
      </form>
    </Form>
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
        <FluidShell
          shell={shell}
          surface={surface}
          background={background}
          text={text}
          radiusBase={radiusBase}
          hasBrand={brandBlocks.length > 0}
          brand={brandBlocks.map((block) => (
            <div key={`brand-${block.id}`} className="text-white">
              {renderNode(block, { wrapperClass: 'text-white' })}
            </div>
          ))}
        >
          {previewForm(shell === 'SplitScreen' ? formBlocks : nonSplitBlocks)}
          {showChrome && (
            <div className="mt-6 text-[10px] text-muted-foreground">
              Click a block to inspect it. Use the + buttons to add blocks.
            </div>
          )}
        </FluidShell>
      </div>
    </section>
  )
}
