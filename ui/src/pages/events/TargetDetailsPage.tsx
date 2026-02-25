import { useMemo, useState } from 'react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent } from '@/components/card'
import { Switch } from '@/components/switch'
import type { DeliveryInspectorItem } from '@/features/events/components/DeliveriesInspector'
import { DeliveriesInspector } from '@/features/events/components/DeliveriesInspector'
import { useDeliveryLogs } from '@/features/events/api/useDeliveryLogs'
import { useWebhookDeliveries } from '@/features/events/api/useWebhookDeliveries'
import { useDeleteWebhook } from '@/features/events/api/useDeleteWebhook'
import { useWebhookMutations } from '@/features/events/api/useWebhookMutations'
import { useWebhook } from '@/features/events/api/useWebhooks'
import { WebhookEndpointForm } from '@/features/events/components/WebhookEndpointForm'
import { useRollWebhookSecret } from '@/features/events/api/useRollWebhookSecret'
import { useReplayDelivery } from '@/features/events/api/useReplayDelivery'
import { usePlugins } from '@/features/plugin/api/usePlugins'
import { usePluginMutations } from '@/features/plugin/api/usePluginMutations'
import { useCurrentRealm } from '@/features/realm/api/useRealm'
import { formatClockTime } from '@/lib/utils'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'
import { Main } from '@/widgets/Layout/Main'
import { ArrowLeft, Copy, Eye, EyeOff, RefreshCcw, RotateCcw, Trash2 } from 'lucide-react'
import { useParams } from 'react-router-dom'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'

export function TargetDetailsPage() {
  const { targetType, targetId } = useParams<{ targetType: string; targetId: string }>()
  const [showSecret, setShowSecret] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const navigate = useRealmNavigate()

  const isWebhook = targetType === 'webhooks'
  const webhookId = isWebhook ? targetId : undefined

  const { data: webhookDetails, isLoading: webhookLoading, refetch: refetchWebhook } = useWebhook(
    webhookId,
  )
  const { data: pluginData } = usePlugins()
  const { data: currentRealm } = useCurrentRealm()
  const { enableWebhook, disableWebhook } = useWebhookMutations()
  const { enablePlugin, disablePlugin } = usePluginMutations()
  const deleteWebhook = useDeleteWebhook()
  const rollSecret = useRollWebhookSecret()
  const replayDelivery = useReplayDelivery()

  const plugin = useMemo(
    () => pluginData?.statuses.find((item) => item.manifest.id === targetId),
    [pluginData, targetId],
  )

  const {
    data: webhookDeliveries,
    isLoading: webhookDeliveriesLoading,
    isFetching: webhookDeliveriesFetching,
    refetch: refetchWebhookDeliveries,
  } = useWebhookDeliveries(
    webhookId,
    { per_page: 50, page: 1 },
  )

  const {
    data: pluginDeliveries,
    isLoading: pluginDeliveriesLoading,
    isFetching: pluginDeliveriesFetching,
    refetch: refetchPluginDeliveries,
  } = useDeliveryLogs(
    {
      target_type: 'plugin',
      target_id: targetId,
      realm_id: currentRealm?.id,
      limit: 50,
    },
    !isWebhook && !!targetId,
  )

  const endpoint = webhookDetails?.endpoint
  const isActive = isWebhook
    ? endpoint?.status === 'active'
    : plugin?.status === 'active'

  const toggleDisabled = isWebhook
    ? !endpoint || enableWebhook.isPending || disableWebhook.isPending || webhookLoading
    : !plugin || enablePlugin.isPending || disablePlugin.isPending

  const profileName = isWebhook
    ? endpoint?.url || endpoint?.name || 'Webhook endpoint'
    : plugin?.manifest.name || 'gRPC plugin'

  const pluginFailed =
    !isWebhook && !!plugin && typeof plugin.status === 'object' && 'failed' in plugin.status

  const statusLabel =
    isActive === undefined
      ? 'Loading'
      : pluginFailed
        ? 'Failed'
        : isActive
          ? 'Active'
          : 'Disabled'
  const statusVariant =
    isActive === undefined
      ? 'muted'
      : pluginFailed
        ? 'destructive'
        : isActive
          ? 'success'
          : 'destructive'

  const maskedSecret = useMemo(() => {
    const secret = endpoint?.signing_secret
    if (!secret) return '—'
    if (showSecret) return secret
    if (secret.length <= 10) return `${secret.slice(0, 4)}****`
    return `${secret.slice(0, 6)}****${secret.slice(-4)}`
  }, [endpoint?.signing_secret, showSecret])

  const deliveries = useMemo<DeliveryInspectorItem[]>(() => {
    const logs = isWebhook ? webhookDeliveries?.data ?? [] : pluginDeliveries?.data ?? []
    return logs.map((log) => {
      const isSuccess =
        typeof log.response_status === 'number' &&
        log.response_status >= 200 &&
        log.response_status < 300 &&
        !log.error

      const payload = parseJsonPayload(log.payload)
      const responseBody = parseJsonPayload(log.response_body ?? undefined)
      const failureReason = formatFailureReason(log.error, log.response_status)
      const errorChain = parseErrorChain(log.error_chain ?? undefined)
      const statusText = formatStatusText(log.response_status, log.error)

      return {
        id: log.id,
        eventType: log.event_type,
        status: isSuccess ? 'success' : 'failed',
        timestamp: formatClockTime(log.delivered_at),
        latency: log.latency_ms ? `${log.latency_ms}ms` : '—',
        signature: null,
        payload,
        failureReason,
        errorChain,
        response: {
          status: statusText,
          body: responseBody,
        },
      }
    })
  }, [isWebhook, pluginDeliveries?.data, webhookDeliveries?.data])

  const deliveriesLoading = isWebhook ? webhookDeliveriesLoading : pluginDeliveriesLoading
  const deliveriesRefreshing = isWebhook ? webhookDeliveriesFetching : pluginDeliveriesFetching
  const refreshDeliveries = () => {
    if (isWebhook) {
      void refetchWebhookDeliveries()
    } else {
      void refetchPluginDeliveries()
    }
  }

  const handleStatusChange = (checked: boolean) => {
    if (isWebhook && endpoint) {
      if (checked) {
        enableWebhook.mutate(endpoint.id)
      } else {
        disableWebhook.mutate({ endpointId: endpoint.id, reason: 'Disabled via UI' })
      }
      return
    }

    if (!isWebhook && plugin) {
      if (checked) {
        enablePlugin.mutate(plugin.manifest.id)
      } else {
        disablePlugin.mutate(plugin.manifest.id)
      }
    }
  }

  const handleCopySecret = async () => {
    const secret = endpoint?.signing_secret
    if (!secret) return
    try {
      await navigator.clipboard.writeText(secret)
    } catch (err) {
      console.error('Failed to copy signing secret', err)
    }
  }

  const handleRollSecret = () => {
    if (!endpoint) return
    rollSecret.mutate(endpoint.id, {
      onSuccess: () => {
        setShowSecret(false)
      },
    })
  }

  const handleDelete = async () => {
    if (!endpoint) return
    try {
      await deleteWebhook.mutateAsync(endpoint.id)
      setDeleteOpen(false)
      navigate('/events')
    } catch (err) {
      console.error('Failed to delete webhook', err)
    }
  }

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <Button variant="ghost" className="w-fit gap-2" onClick={() => navigate('/events')}>
        <ArrowLeft className="h-4 w-4" />
        Back to Event Routing
      </Button>

      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-semibold tracking-tight">{profileName}</h1>
            <Badge variant={statusVariant}>
              {statusLabel}
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            {isWebhook
              ? 'Securely deliver events to this endpoint with signed payloads.'
              : 'Inspect delivery history for the selected gRPC plugin.'}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <Button
            variant="outline"
            onClick={refreshDeliveries}
            disabled={deliveriesRefreshing || deliveriesLoading}
          >
            <RefreshCcw className="h-4 w-4" />
          </Button>
          {isWebhook && endpoint && (
            <>
              <WebhookEndpointForm
                mode="edit"
                endpointId={endpoint.id}
                initialUrl={endpoint.url}
                initialMethod={endpoint.http_method}
                initialDescription={endpoint.description}
                initialSubscriptions={(webhookDetails?.subscriptions ?? [])
                  .filter((sub) => sub.enabled)
                  .map((sub) => sub.event_type)}
                onSaved={() => {
                  void refetchWebhook()
                  void refetchWebhookDeliveries()
                }}
                trigger={<Button variant="secondary">Edit</Button>}
              />
              <Button
                variant="destructive"
                onClick={() => setDeleteOpen(true)}
                disabled={deleteWebhook.isPending}
              >
                <Trash2 className="h-4 w-4" />
                Delete
              </Button>
            </>
          )}
          <div className="flex items-center gap-2 rounded-full border px-3 py-2 text-xs text-muted-foreground">
            <span>Status</span>
            <Switch
              checked={!!isActive}
              onCheckedChange={handleStatusChange}
              disabled={toggleDisabled}
            />
          </div>
        </div>
      </div>

      {isWebhook && (
        <Card>
          <CardContent className="flex flex-wrap items-center justify-between gap-4 p-4">
            <div>
              <p className="text-sm font-semibold">Signing Secret</p>
              <p className="text-xs text-muted-foreground">
                Used to verify the integrity of webhook payloads.
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <div className="rounded-md border bg-muted/40 px-3 py-2 font-mono text-xs">
                {maskedSecret}
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setShowSecret((prev) => !prev)}
                disabled={!endpoint?.signing_secret}
              >
                {showSecret ? <EyeOff /> : <Eye />}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={handleCopySecret}
                disabled={!endpoint?.signing_secret}
              >
                <Copy />
              </Button>
              <Button
                variant="secondary"
                onClick={handleRollSecret}
                disabled={!endpoint?.signing_secret || rollSecret.isPending}
              >
                <RotateCcw className="h-4 w-4" />
                Roll Secret
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      <DeliveriesInspector
        deliveries={deliveries}
        isLoading={deliveriesLoading}
        replayPending={replayDelivery.isPending}
        onReplay={(deliveryId) => replayDelivery.mutate(deliveryId)}
      />

      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title="Delete webhook endpoint"
        desc="This will permanently delete the webhook and its delivery history."
        destructive
        isLoading={deleteWebhook.isPending}
        handleConfirm={handleDelete}
        confirmText={deleteWebhook.isPending ? 'Deleting...' : 'Delete'}
      />
    </Main>
  )
}

export default TargetDetailsPage

function parseJsonPayload(value?: string) {
  if (!value) return {}
  const trimmed = value.trim()
  if (!trimmed) return {}
  if (
    (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
    (trimmed.startsWith('[') && trimmed.endsWith(']'))
  ) {
    try {
      return JSON.parse(trimmed)
    } catch {
      return value
    }
  }
  return value
}

function formatStatusText(status?: number | null, error?: string | null) {
  if (typeof status === 'number') {
    return `${status} ${httpStatusText(status)}`
  }
  if (error) return 'Delivery Failed'
  return 'No Response'
}

function formatFailureReason(error?: string | null, status?: number | null) {
  if (error) {
    if (error.startsWith('http_')) {
      return error.replace('http_', 'HTTP ').toUpperCase()
    }
    return error
  }
  if (typeof status === 'number' && (status < 200 || status >= 300)) {
    return `HTTP ${status}`
  }
  return null
}

function parseErrorChain(value?: string | null) {
  if (!value) return null
  try {
    const parsed = JSON.parse(value)
    if (Array.isArray(parsed)) {
      return parsed.map((entry) => String(entry))
    }
  } catch {
    return [value]
  }
  return [value]
}

function httpStatusText(status: number) {
  switch (status) {
    case 200:
      return 'OK'
    case 201:
      return 'Created'
    case 202:
      return 'Accepted'
    case 204:
      return 'No Content'
    case 400:
      return 'Bad Request'
    case 401:
      return 'Unauthorized'
    case 403:
      return 'Forbidden'
    case 404:
      return 'Not Found'
    case 409:
      return 'Conflict'
    case 422:
      return 'Unprocessable Entity'
    case 429:
      return 'Too Many Requests'
    case 500:
      return 'Internal Server Error'
    case 502:
      return 'Bad Gateway'
    case 503:
      return 'Service Unavailable'
    case 504:
      return 'Gateway Timeout'
    default:
      return 'Unknown'
  }
}
