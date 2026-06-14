import { format } from 'date-fns'
import {
  CalendarClock,
  Copy,
  Fingerprint,
  Link,
  ListChecks,
  RadioTower,
  RotateCcw,
} from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import type { WebhookEndpointDetails } from '@/entities/events/model/types'

interface WebhookTargetSummaryPanelProps {
  details: WebhookEndpointDetails
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

export function WebhookTargetSummaryPanel({ details }: WebhookTargetSummaryPanelProps) {
  const { endpoint, subscriptions } = details
  const enabledSubscriptions = subscriptions.filter((subscription) => subscription.enabled)

  return (
    <div className="flex flex-col">
      <SummaryRow icon={Fingerprint} label="Webhook ID" value={endpoint.id} copyable />
      <SummaryRow icon={Link} label="Destination URL" value={endpoint.url} copyable />
      <SummaryRow icon={RadioTower} label="HTTP method" value={endpoint.http_method || 'POST'} />
      <SummaryRow
        icon={ListChecks}
        label="Enabled events"
        value={`${enabledSubscriptions.length} of ${subscriptions.length}`}
      />
      <SummaryRow
        icon={RotateCcw}
        label="Consecutive failures"
        value={String(endpoint.consecutive_failures)}
      />
      <SummaryRow
        icon={CalendarClock}
        label="Last fired"
        value={formatDate(endpoint.last_fired_at)}
      />
      <SummaryRow icon={CalendarClock} label="Created" value={formatDate(endpoint.created_at)} />
      <SummaryRow icon={CalendarClock} label="Updated" value={formatDate(endpoint.updated_at)} />
    </div>
  )
}
