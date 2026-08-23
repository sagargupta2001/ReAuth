import type { LucideIcon } from 'lucide-react'

import { Label } from '@/shared/ui/label'

interface FieldLabelProps {
  htmlFor?: string
  label: string
  icon?: LucideIcon
  hint?: string
}

/** Consistent label (and optional hint) for every settings field. */
export function FieldLabel({ htmlFor, label, icon: Icon, hint }: FieldLabelProps) {
  return (
    <div className="space-y-1">
      <Label htmlFor={htmlFor} className="flex items-center gap-2 text-xs">
        {Icon && <Icon className="text-muted-foreground h-3.5 w-3.5" />}
        {label}
      </Label>
      {hint && <p className="text-muted-foreground text-[10px]">{hint}</p>}
    </div>
  )
}
