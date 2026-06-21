import { Copy, Fingerprint, Globe, Hash, KeyRound, ShieldCheck } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import type { OidcClient } from '@/entities/oidc/model/types.ts'
import { parseJsonArray } from '@/features/client/lib/clientFields'

interface ClientSummaryPanelProps {
  client: OidcClient
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

export function ClientSummaryPanel({ client }: ClientSummaryPanelProps) {
  const isConfidential = client.confidential ?? !!client.client_secret
  const scopes = parseJsonArray(client.scopes)
  const redirectUris = parseJsonArray(client.redirect_uris)

  return (
    <div className="flex flex-col self-start xl:sticky xl:top-6">
      <SummaryRow icon={Fingerprint} label="Internal ID" value={client.id} copyable />
      <SummaryRow icon={Hash} label="Client ID" value={client.client_id} copyable />
      <SummaryRow
        icon={ShieldCheck}
        label="Type"
        value={isConfidential ? 'Confidential' : 'Public'}
      />
      <SummaryRow
        icon={KeyRound}
        label="Scopes"
        value={scopes.length ? scopes.join(', ') : '-'}
      />
      <SummaryRow
        icon={Globe}
        label="Redirect URIs"
        value={String(redirectUris.length)}
      />
    </div>
  )
}
