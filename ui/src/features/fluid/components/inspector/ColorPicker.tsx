import { Input } from '@/components/input'
import { normalizeColorValue } from '@/shared/lib/colorUtils'

interface ColorPickerProps {
  id?: string
  value: string
  onChange: (value: string) => void
  disabled?: boolean
}

export function ColorPicker({ id, value, onChange, disabled }: ColorPickerProps) {
  return (
    <div className="flex items-center gap-2">
      <input
        type="color"
        aria-label="Font color"
        className="h-8 w-8 cursor-pointer rounded-md border bg-transparent p-0"
        value={normalizeColorValue(value || '#111827')}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
      <Input
        id={id}
        value={value || ''}
        placeholder="#111827"
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
    </div>
  )
}
