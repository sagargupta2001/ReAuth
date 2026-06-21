import type { ReactNode } from 'react'

import { format } from 'date-fns'
import {
  CalendarClock,
  CalendarPlus,
  Copy,
  Fingerprint,
  GitBranch,
  Lock,
  type LucideIcon,
  Workflow,
} from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import type { FlowDraft } from '@/entities/flow/model/types'

interface FlowSummaryPanelProps {
  draft: FlowDraft
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
  icon: LucideIcon
  label: string
  value: ReactNode
  copyable?: boolean
}) {
  const canCopy = copyable && typeof value === 'string' && value !== '-'

  const copyValue = () => {
    if (!canCopy) return

    void navigator.clipboard
      .writeText(value)
      .then(() => toast.success(`${label} copied.`))
      .catch(() => toast.error(`Failed to copy ${label.toLowerCase()}.`))
  }

  return (
    // border-b puts bottom borders everywhere; last:border-b-0 removes it from the final row
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

export function FlowSummaryPanel({ draft }: FlowSummaryPanelProps) {
  return (
    // self-start shields it from expanding layout constraints
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="Flow ID" value={draft.id} copyable />
      <SummaryRow
        icon={Workflow}
        label="Flow type"
        value={<span className="capitalize">{draft.flow_type}</span>}
      />
      <SummaryRow
        icon={draft.built_in ? Lock : GitBranch}
        label="Origin"
        value={
          <Badge variant={draft.built_in ? 'secondary' : 'outline'} className="h-5 text-[10px]">
            {draft.built_in ? 'System' : 'Custom'}
          </Badge>
        }
      />
      <SummaryRow
        icon={GitBranch}
        label="Active version"
        value={
          draft.active_version ? (
            <Badge variant="secondary" className="h-5 text-[10px]">
              v{draft.active_version}
            </Badge>
          ) : (
            <Badge variant="outline" className="h-5 text-[10px]">
              Unpublished draft
            </Badge>
          )
        }
      />
      <SummaryRow icon={CalendarPlus} label="Created" value={formatDate(draft.created_at)} />
      <SummaryRow icon={CalendarClock} label="Last updated" value={formatDate(draft.updated_at)} />
    </div>
  )
}
