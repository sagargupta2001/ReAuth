import { Ruler } from 'lucide-react'

import { Input } from '@/components/input'
import { SectionCard } from '@/shared/ui/section-card'
import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'

interface SpacingControlsProps {
  padding: string
  marginTop: string
  marginBottom: string
  disabled?: boolean
  onChange: (patch: { padding?: string; margin_top?: string; margin_bottom?: string }) => void
}

export function SpacingControls({
  padding,
  marginTop,
  marginBottom,
  disabled,
  onChange,
}: SpacingControlsProps) {
  return (
    <SectionCard title="Spacing" description="Padding and margins.">
      <div className="space-y-2">
        <FieldLabel htmlFor="spacing-padding" label="Padding" icon={Ruler} />
        <Input
          id="spacing-padding"
          value={padding}
          disabled={disabled}
          onChange={(event) => onChange({ padding: event.target.value })}
        />
      </div>
      <div className="space-y-2">
        <FieldLabel htmlFor="spacing-margin-top" label="Margin Top" icon={Ruler} />
        <Input
          id="spacing-margin-top"
          value={marginTop}
          disabled={disabled}
          onChange={(event) => onChange({ margin_top: event.target.value })}
        />
      </div>
      <div className="space-y-2">
        <FieldLabel htmlFor="spacing-margin-bottom" label="Margin Bottom" icon={Ruler} />
        <Input
          id="spacing-margin-bottom"
          value={marginBottom}
          disabled={disabled}
          onChange={(event) => onChange({ margin_bottom: event.target.value })}
        />
      </div>
    </SectionCard>
  )
}
