import { Type } from 'lucide-react'

import { Input } from '@/components/input'
import { Label } from '@/shared/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { ColorPicker } from './ColorPicker'

interface TypographyControlsProps {
  fontSize: string
  fontWeight: string
  color: string
  disabled?: boolean
  onChange: (patch: { font_size?: string; font_weight?: string; color?: string }) => void
}

export function TypographyControls({
  fontSize,
  fontWeight,
  color,
  disabled,
  onChange,
}: TypographyControlsProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Typography</CardTitle>
        <CardDescription>Font overrides for this block.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Type className="h-3.5 w-3.5" />
          <span>Typography</span>
        </div>
        <div className="space-y-2">
          <Label htmlFor="font-size">Font Size</Label>
          <Input
            id="font-size"
            value={fontSize}
            placeholder="e.g. 16px"
            disabled={disabled}
            onChange={(event) => onChange({ font_size: event.target.value })}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="font-weight">Font Weight</Label>
          <Input
            id="font-weight"
            value={fontWeight}
            placeholder="e.g. 600 or bold"
            disabled={disabled}
            onChange={(event) => onChange({ font_weight: event.target.value })}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="font-color">Color</Label>
          <ColorPicker
            id="font-color"
            value={color}
            disabled={disabled}
            onChange={(newColor) => onChange({ color: newColor })}
          />
        </div>
      </CardContent>
    </Card>
  )
}
