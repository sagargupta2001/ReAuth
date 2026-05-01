import { Ruler } from 'lucide-react'

import { Input } from '@/components/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'

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
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Spacing</CardTitle>
        <CardDescription>Padding and margins.</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-3">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Ruler className="h-3.5 w-3.5" />
          <span>Padding</span>
        </div>
        <Input
          value={padding}
          disabled={disabled}
          onChange={(event) => onChange({ padding: event.target.value })}
        />
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Ruler className="h-3.5 w-3.5" />
          <span>Margin Top</span>
        </div>
        <Input
          value={marginTop}
          disabled={disabled}
          onChange={(event) => onChange({ margin_top: event.target.value })}
        />
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Ruler className="h-3.5 w-3.5" />
          <span>Margin Bottom</span>
        </div>
        <Input
          value={marginBottom}
          disabled={disabled}
          onChange={(event) => onChange({ margin_bottom: event.target.value })}
        />
      </CardContent>
    </Card>
  )
}
