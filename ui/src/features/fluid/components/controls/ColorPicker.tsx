import { Input } from '@/components/input'
import { TOKEN_FALLBACK } from '@/features/fluid/model/tokens'
import { normalizeColorValue } from '@/lib/colorUtils'

interface ColorPickerProps {
  id?: string
  value: string
  onChange: (value: string) => void
  disabled?: boolean
  /** Describes the swatch for assistive tech, e.g. "Primary color". */
  ariaLabel?: string
  /** Swatch colour used while `value` is empty or not a valid hex. */
  fallback?: string
}

/** Native colour swatch paired with a free-text field for hex/var() values. */
export function ColorPicker({
  id,
  value,
  onChange,
  disabled,
  ariaLabel = 'Color',
  fallback = TOKEN_FALLBACK.color,
}: ColorPickerProps) {
  return (
    <div className="flex items-center gap-2">
      <input
        type="color"
        aria-label={ariaLabel}
        className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
        value={normalizeColorValue(value || fallback)}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
      <Input
        id={id}
        value={value || ''}
        placeholder={fallback}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
    </div>
  )
}
