import { json } from '@codemirror/lang-json'
import { oneDark } from '@codemirror/theme-one-dark'
import CodeMirror from '@uiw/react-codemirror'
import { format } from 'date-fns'
import { Copy } from 'lucide-react'
import { toast } from 'sonner'

import {
  deriveSessionStatus,
  parseUserAgent,
  sessionTypeLabel,
  statusBadge,
} from '@/entities/session/lib/session.logic'
import type { Session } from '@/entities/session/model/types'
import { Badge } from '@/shared/ui/badge.tsx'
import { Button } from '@/shared/ui/button.tsx'
import { Separator } from '@/shared/ui/separator'
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/shared/ui/sheet.tsx'

interface SessionDetailsDrawerProps {
  session: Session | null
  currentSessionId: string | undefined
  open: boolean
  onOpenChange: (open: boolean) => void
}

function fmt(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '—' : format(date, 'PPpp')
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-0.5 py-2">
      <span className="text-muted-foreground text-xs uppercase tracking-wide">{label}</span>
      <span className="text-sm break-all">{children}</span>
    </div>
  )
}

export function SessionDetailsDrawer({
  session,
  currentSessionId,
  open,
  onOpenChange,
}: SessionDetailsDrawerProps) {
  const value = JSON.stringify(session ?? {}, null, 2)

  const copyJson = () => {
    navigator.clipboard
      .writeText(value)
      .then(() => toast.success('Session JSON copied.'))
      .catch(() => toast.error('Failed to copy session JSON.'))
  }

  const device = parseUserAgent(session?.user_agent)
  const badge = session ? statusBadge(deriveSessionStatus(session, currentSessionId)) : null

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="right"
        overlayClassName="bg-background/80 dot-grid text-muted-foreground/20"
        className="flex w-full flex-col gap-0 sm:max-w-md"
      >
        <SheetHeader>
          <SheetTitle>Session details</SheetTitle>
        </SheetHeader>

        {session ? (
          <div className="flex-1 overflow-y-auto px-1">
            <div className="flex flex-wrap items-center gap-2 py-2">
              <Badge variant="outline">{sessionTypeLabel(session)}</Badge>
              {badge && <Badge variant={badge.variant}>{badge.label}</Badge>}
            </div>

            <Separator />

            <Row label="Session ID">
              <span className="font-mono text-xs">{session.id}</span>
            </Row>
            <Row label="Token family">
              <span className="font-mono text-xs">{session.family_id ?? '—'}</span>
            </Row>
            <Row label="User">
              {session.username || 'Unknown user'}
              {session.email && (
                <span className="text-muted-foreground mt-0.5 block text-xs">{session.email}</span>
              )}
              <span className="text-muted-foreground mt-0.5 block font-mono text-[11px]">
                {session.user_id}
              </span>
            </Row>
            <Row label="Client">
              {session.client_id ? (
                <span className="font-mono text-xs">{session.client_id}</span>
              ) : (
                <span className="text-muted-foreground italic">Admin Console (browser SSO)</span>
              )}
            </Row>

            <Separator />

            <Row label="Device">
              {device.label}
              {session.user_agent && (
                <span className="text-muted-foreground mt-1 block font-mono text-[11px]">
                  {session.user_agent}
                </span>
              )}
            </Row>
            <Row label="IP address">
              <span className="font-mono text-xs">{session.ip_address || 'Unknown'}</span>
            </Row>

            <Separator />

            <Row label="Issued at (iat)">{fmt(session.created_at)}</Row>
            <Row label="Expires at (exp)">{fmt(session.expires_at)}</Row>
            <Row label="Last used">{fmt(session.last_used_at)}</Row>
            <Row label="Re-auth requested">
              {session.step_up_at ? fmt(session.step_up_at) : 'No'}
            </Row>

            <Separator className="my-2" />

            <div className="flex items-center justify-between py-2">
              <span className="text-muted-foreground text-xs uppercase tracking-wide">
                Raw token JSON
              </span>
              <Button size="sm" variant="outline" onClick={copyJson}>
                <Copy className="mr-2 h-3.5 w-3.5" />
                Copy
              </Button>
            </div>
            <div className="overflow-hidden rounded-lg border">
              <CodeMirror
                value={value}
                height="240px"
                theme={oneDark}
                extensions={[json()]}
                basicSetup={{ foldGutter: true, lineNumbers: true }}
                editable={false}
              />
            </div>
          </div>
        ) : null}
      </SheetContent>
    </Sheet>
  )
}
