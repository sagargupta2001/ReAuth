import { format } from 'date-fns'
import { CalendarClock, Copy, Fingerprint } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import type { Role } from '@/features/roles/api/useRoles'

interface RoleSummaryPanelProps {
  role: Role
}

function formatDate(value?: string | null) {
  if (!value) return '-'
  return format(new Date(value), 'MMM d, yyyy, h:mm a')
}

function SummaryRow({
  icon: Icon,
  label,
  value,
  copyable = false,
}: {
  icon: typeof Fingerprint
  label: string
  value: string
  copyable?: boolean
}) {
  const canCopy = copyable && value !== '-'

  const copyValue = () => {
    if (!canCopy) return

    void navigator.clipboard
      .writeText(value)
      .then(() => toast.success(`${label} copied.`))
      .catch(() => toast.error(`Failed to copy ${label.toLowerCase()}.`))
  }

  return (
    <div className="border-border/60 border-b py-3 last:border-b-0">
      <div className="text-muted-foreground mb-1 flex items-center gap-2 text-xs font-medium">
        <Icon className="h-3.5 w-3.5" />
        {label}
      </div>
      <div className="flex min-w-0 items-center justify-between gap-2">
        <span className="truncate text-sm font-medium">{value}</span>
        {copyable ? (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 shrink-0"
            disabled={!canCopy}
            onClick={copyValue}
            aria-label={`Copy ${label}`}
          >
            <Copy className="h-3.5 w-3.5" />
          </Button>
        ) : null}
      </div>
    </div>
  )
}

export function RoleSummaryPanel({ role }: RoleSummaryPanelProps) {
  return (
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="Role ID" value={role.id} copyable />
      <SummaryRow icon={CalendarClock} label="Role created on" value={formatDate(role.created_at)} />
    </div>
  )
}
