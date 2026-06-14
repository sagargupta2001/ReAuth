import { useMemo, useState } from 'react'

import { Copy, Eye, EyeOff, RotateCcw, Trash2 } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Switch } from '@/components/switch'
import type { WebhookEndpoint } from '@/entities/events/model/types'
import { useRollWebhookSecret } from '@/features/events/api/useRollWebhookSecret'

interface WebhookSettingsTabProps {
  endpoint: WebhookEndpoint
  statusPending: boolean
  deletePending: boolean
  onStatusChange: (enabled: boolean) => void
  onDeleteClick: () => void
}

export function WebhookSettingsTab({
  endpoint,
  statusPending,
  deletePending,
  onStatusChange,
  onDeleteClick,
}: WebhookSettingsTabProps) {
  const [showSecret, setShowSecret] = useState(false)
  const rollSecret = useRollWebhookSecret()
  const isActive = endpoint.status === 'active'

  const maskedSecret = useMemo(() => {
    const secret = endpoint.signing_secret
    if (!secret) return '-'
    if (showSecret) return secret
    if (secret.length <= 10) return `${secret.slice(0, 4)}****`
    return `${secret.slice(0, 6)}****${secret.slice(-4)}`
  }, [endpoint.signing_secret, showSecret])

  const handleCopySecret = async () => {
    if (!endpoint.signing_secret) return
    try {
      await navigator.clipboard.writeText(endpoint.signing_secret)
    } catch (err) {
      console.error('Failed to copy signing secret', err)
    }
  }

  const handleRollSecret = () => {
    rollSecret.mutate(endpoint.id, {
      onSuccess: () => setShowSecret(false),
    })
  }

  return (
    <div className="max-w-4xl space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Delivery State</CardTitle>
          <CardDescription>Enable or disable delivery to this webhook target.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-primary-foreground flex items-center justify-between gap-4 rounded-2xl p-4">
            <div>
              <p className="text-sm font-medium">{isActive ? 'Enabled' : 'Disabled'}</p>
              <p className="text-muted-foreground text-sm">
                Disabled targets keep their configuration but do not receive new events.
              </p>
            </div>
            <Switch checked={isActive} onCheckedChange={onStatusChange} disabled={statusPending} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Signing Secret</CardTitle>
          <CardDescription>
            Consumers use this secret to verify the integrity of webhook payloads.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-primary-foreground space-y-4 rounded-2xl p-4">
            <div className="bg-muted/40 rounded-md border px-3 py-2 font-mono text-xs">
              {maskedSecret}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setShowSecret((prev) => !prev)}
                disabled={!endpoint.signing_secret}
              >
                {showSecret ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                {showSecret ? 'Hide Secret' : 'Reveal Secret'}
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleCopySecret}
                disabled={!endpoint.signing_secret}
              >
                <Copy className="h-4 w-4" />
                Copy Secret
              </Button>
              <Button
                type="button"
                variant="secondary"
                size="sm"
                onClick={handleRollSecret}
                disabled={!endpoint.signing_secret || rollSecret.isPending}
              >
                <RotateCcw className="h-4 w-4" />
                {rollSecret.isPending ? 'Rolling...' : 'Roll Secret'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Danger Zone</CardTitle>
          <CardDescription>
            Delete this webhook endpoint and remove its event subscriptions.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="border-destructive/30 bg-destructive/5 flex flex-wrap items-center justify-between gap-4 rounded-2xl border p-4">
            <div>
              <p className="text-sm font-medium">Delete webhook endpoint</p>
              <p className="text-muted-foreground text-sm">
                This permanently removes the target from event routing.
              </p>
            </div>
            <Button
              type="button"
              variant="destructive"
              onClick={onDeleteClick}
              disabled={deletePending}
            >
              <Trash2 className="h-4 w-4" />
              Delete Webhook
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
