import { Input } from '@/components/input'
import { Textarea } from '@/components/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'
import { IconPicker } from '@/features/fluid/components/inspector/IconPicker'
import { InputSlotsPanel } from '@/features/fluid/components/inspector/InputSlotsPanel'
import { useInspector } from '@/features/fluid/components/inspector/inspectorContext'
import {
  CustomInspectorPanel,
  FieldTarget,
  InspectorFieldKind,
  type InspectorField as Field,
} from '@/features/fluid/model/inspectorFields'

/**
 * Renders one schema field.
 *
 * The union is exhaustive, so adding an `InspectorFieldKind` without handling it
 * here fails to compile.
 */
export function InspectorField({ field }: { field: Field }) {
  const { node, read, write, assets } = useInspector()

  switch (field.kind) {
    case InspectorFieldKind.Readonly:
      return (
        <Labelled field={field}>
          <Input id={field.id} value={field.value(node)} readOnly disabled />
        </Labelled>
      )

    case InspectorFieldKind.Text:
      return (
        <Labelled field={field}>
          <Input
            id={field.id}
            value={asText(read(field.target, field.key))}
            placeholder={field.placeholder}
            onChange={(event) => write(field.target, field.key, event.target.value)}
          />
        </Labelled>
      )

    case InspectorFieldKind.Number:
      return (
        <Labelled field={field}>
          <Input
            id={field.id}
            type="number"
            min={field.min}
            max={field.max}
            step={field.step}
            placeholder={field.placeholder}
            value={asText(read(field.target, field.key))}
            onChange={(event) => {
              const raw = event.target.value
              write(field.target, field.key, raw === '' ? '' : Number(raw))
            }}
          />
        </Labelled>
      )

    case InspectorFieldKind.Textarea:
      return (
        <Labelled field={field}>
          <Textarea
            id={field.id}
            className="min-h-[80px] text-xs"
            placeholder={field.placeholder}
            value={asText(read(field.target, field.key))}
            onChange={(event) => write(field.target, field.key, event.target.value)}
          />
        </Labelled>
      )

    case InspectorFieldKind.Select:
      return (
        <Labelled field={field}>
          <Select
            value={asText(read(field.target, field.key)) || field.fallback}
            onValueChange={(value) => write(field.target, field.key, value)}
          >
            <SelectTrigger id={field.id}>
              <SelectValue placeholder={field.label} />
            </SelectTrigger>
            <SelectContent>
              {field.options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Labelled>
      )

    case InspectorFieldKind.Icon:
      return (
        <Labelled field={field}>
          <div className="flex gap-2">
            <Input
              id={field.id}
              value={asText(read(field.target, field.key))}
              onChange={(event) => write(field.target, field.key, event.target.value)}
            />
            <IconPicker
              value={asText(read(field.target, field.key))}
              color={asText(read(FieldTarget.Props, 'color'))}
              onSelect={(name) => write(field.target, field.key, name)}
            />
          </div>
        </Labelled>
      )

    case InspectorFieldKind.Asset:
      return (
        <Labelled field={field}>
          <Select
            value={asText(read(field.target, field.key))}
            onValueChange={(value) => write(field.target, field.key, value)}
          >
            <SelectTrigger id={field.id}>
              <SelectValue placeholder="Select asset" />
            </SelectTrigger>
            <SelectContent>
              {assets.length === 0 ? (
                <SelectItem value="none" disabled>
                  No assets uploaded
                </SelectItem>
              ) : (
                assets.map((asset) => (
                  <SelectItem key={asset.id} value={asset.id}>
                    {asset.filename}
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </Labelled>
      )

    case InspectorFieldKind.PaddingBox:
      return <PaddingBoxField field={field} />

    case InspectorFieldKind.Dimension:
      return <DimensionField field={field} />

    case InspectorFieldKind.Custom:
      switch (field.panel) {
        case CustomInspectorPanel.InputSlots:
          return <InputSlotsPanel />
        default:
          return assertNever(field.panel)
      }

    default:
      return assertNever(field)
  }
}

function Labelled({
  field,
  children,
}: {
  field: Extract<Field, { label: string }>
  children: React.ReactNode
}) {
  return (
    <div className="space-y-2">
      <FieldLabel htmlFor={field.id} label={field.label} hint={field.hint} />
      {children}
    </div>
  )
}

/** The four-number `layout.padding` tuple, edited as one field per side. */
function PaddingBoxField({
  field,
}: {
  field: Extract<Field, { kind: typeof InspectorFieldKind.PaddingBox }>
}) {
  const { read, write } = useInspector()
  const raw = read(FieldTarget.Layout, field.key)
  const values = Array.isArray(raw) ? raw.map((value) => Number(value) || 0) : [0, 0, 0, 0]
  const sides = ['Top', 'Right', 'Bottom', 'Left'] as const

  return (
    <div className="space-y-2">
      <FieldLabel label={field.label} hint={field.hint} />
      <div className="grid grid-cols-4 gap-2">
        {sides.map((side, index) => (
          <Input
            key={side}
            type="number"
            min={0}
            aria-label={`${field.label} ${side.toLowerCase()}`}
            placeholder={side}
            value={String(values[index] ?? 0)}
            onChange={(event) => {
              // Always write all four: a partial tuple is not a valid padding.
              const next = [...values]
              next[index] = Number(event.target.value) || 0
              write(FieldTarget.Layout, field.key, next.slice(0, 4))
            }}
          />
        ))}
      </div>
    </div>
  )
}

/** A width or height mode, with its explicit value shown only when fixed. */
function DimensionField({
  field,
}: {
  field: Extract<Field, { kind: typeof InspectorFieldKind.Dimension }>
}) {
  const { read, write } = useInspector()
  const modeKey = field.axis
  const valueKey = `${field.axis}_value`
  const mode = asText(read(FieldTarget.Size, modeKey)) || field.fallback

  return (
    <div className="space-y-2">
      <FieldLabel htmlFor={field.id} label={field.label} hint={field.hint} />
      <Select value={mode} onValueChange={(value) => write(FieldTarget.Size, modeKey, value)}>
        <SelectTrigger id={field.id}>
          <SelectValue placeholder={field.label} />
        </SelectTrigger>
        <SelectContent>
          {field.options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {mode === 'fixed' && (
        <Input
          id={`${field.id}-value`}
          aria-label={field.valueLabel}
          // A bare number is coerced to px by the renderer, so both work.
          placeholder="e.g. 240px"
          value={asText(read(FieldTarget.Size, valueKey))}
          onChange={(event) => write(FieldTarget.Size, valueKey, event.target.value)}
        />
      )}
    </div>
  )
}

function asText(value: unknown): string {
  if (value === undefined || value === null) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'number') return String(value)
  return ''
}

function assertNever(value: never): never {
  throw new Error(`Unhandled inspector field: ${JSON.stringify(value)}`)
}
