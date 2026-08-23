import { Input } from '@/components/input'
import { Textarea } from '@/components/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import type { ThemeNode } from '@/entities/theme/model/types'
import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'
import { IconPicker } from '@/features/fluid/components/inspector/IconPicker'
import { useInspector } from '@/features/fluid/components/inspector/inspectorContext'
import { createNodeFromDefinition } from '@/features/fluid/lib/nodeUtils'

const VISIBILITY_OPTIONS = [
  { value: 'show', label: 'Show' },
  { value: 'hide', label: 'Hide' },
] as const

/**
 * Editors for an Input component's `prefix` and `error` slots.
 *
 * Slots hold whole child nodes rather than a single value, so this stays a
 * bespoke panel referenced from the schema by `CustomInspectorPanel.InputSlots`
 * instead of being forced into a field kind.
 */
export function InputSlotsPanel() {
  const { node, patch } = useInspector()
  const prefixSlot = node.slots?.prefix
  const errorSlot = node.slots?.error

  /** Merges props into a slot, creating the slot node if it does not exist. */
  const upsertSlot = (
    key: string,
    baseType: ThemeNode['type'],
    props: Record<string, unknown>,
  ) => {
    const base = node.slots?.[key] ?? createNodeFromDefinition({ type: baseType, props })
    patch({
      slots: { [key]: { ...base, props: { ...(base.props ?? {}), ...props } } },
    })
  }

  const prefixVisible = Boolean(prefixSlot) && (prefixSlot?.props?.visible ?? true) !== false
  const errorVisible = Boolean(errorSlot) && Boolean(errorSlot?.props?.visible ?? false)

  // Editing any prefix field implies showing it; hiding is the explicit toggle.
  const setPrefix = (props: Record<string, unknown>) =>
    upsertSlot('prefix', 'Icon', { ...props, visible: true })
  const setError = (props: Record<string, unknown>) =>
    upsertSlot('error', 'Text', { ...props, visible: true })

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <FieldLabel label="Prefix Icon" />
        <Select
          value={prefixVisible ? 'show' : 'hide'}
          onValueChange={(value) =>
            upsertSlot('prefix', 'Icon', { visible: value === 'show' })
          }
        >
          <SelectTrigger aria-label="Prefix icon visibility">
            <SelectValue placeholder="Toggle prefix icon" />
          </SelectTrigger>
          <SelectContent>
            {VISIBILITY_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div className="flex gap-2">
          <Input
            aria-label="Prefix icon name"
            value={String(prefixSlot?.props?.name || '')}
            placeholder="Icon name"
            onChange={(event) => setPrefix({ name: event.target.value })}
          />
          <IconPicker
            value={String(prefixSlot?.props?.name || '')}
            color={String(prefixSlot?.props?.color || '')}
            onSelect={(name) => setPrefix({ name })}
          />
        </div>
        <div className="grid grid-cols-2 gap-2">
          <Input
            aria-label="Prefix icon size"
            value={String(prefixSlot?.props?.size || '')}
            placeholder="Size"
            onChange={(event) => setPrefix({ size: event.target.value })}
          />
          <Input
            aria-label="Prefix icon color"
            value={String(prefixSlot?.props?.color || '')}
            placeholder="Color"
            onChange={(event) => setPrefix({ color: event.target.value })}
          />
        </div>
        <Textarea
          aria-label="Prefix icon SVG path"
          value={String(prefixSlot?.props?.svg_path || '')}
          placeholder="SVG path (d attribute)"
          className="min-h-[80px] text-xs"
          onChange={(event) => setPrefix({ svg_path: event.target.value })}
        />
        <Input
          aria-label="Prefix icon viewBox"
          value={String(prefixSlot?.props?.svg_viewbox || '')}
          placeholder="ViewBox (e.g. 0 0 24 24)"
          onChange={(event) => setPrefix({ svg_viewbox: event.target.value })}
        />
        <p className="text-muted-foreground text-[10px]">
          When an SVG path is provided, it overrides the icon name.
        </p>
      </div>

      <div className="space-y-2">
        <FieldLabel label="Error Hint" />
        <Select
          value={errorVisible ? 'show' : 'hide'}
          onValueChange={(value) => upsertSlot('error', 'Text', { visible: value === 'show' })}
        >
          <SelectTrigger aria-label="Error hint visibility">
            <SelectValue placeholder="Toggle error hint" />
          </SelectTrigger>
          <SelectContent>
            {VISIBILITY_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="grid grid-cols-2 gap-2">
          <Input
            aria-label="Error text"
            value={String(errorSlot?.props?.text || '')}
            placeholder="Error text"
            onChange={(event) => setError({ text: event.target.value })}
          />
          <Input
            aria-label="Error color"
            value={String(errorSlot?.props?.color || '')}
            placeholder="Color"
            onChange={(event) => setError({ color: event.target.value })}
          />
        </div>
      </div>
    </div>
  )
}
