import { type ReactNode, useEffect, useMemo, useState } from 'react'

import { Activity, ArrowLeft, Loader2, Settings, SlidersHorizontal } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button, buttonVariants } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import type { WebhookEndpointDetails } from '@/entities/events/model/types'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useDeleteWebhook } from '@/features/events/api/useDeleteWebhook'
import { useReplayDelivery } from '@/features/events/api/useReplayDelivery'
import { useWebhookDeliveries } from '@/features/events/api/useWebhookDeliveries'
import { useWebhookMutations } from '@/features/events/api/useWebhookMutations'
import { useWebhook } from '@/features/events/api/useWebhooks'
import {
  DeliveriesInspector,
  type DeliveryInspectorItem,
} from '@/features/events/components/DeliveriesInspector'
import { WebhookConfigureTab } from '@/features/events/components/WebhookConfigureTab'
import { WebhookSettingsTab } from '@/features/events/components/WebhookSettingsTab'
import { WebhookTargetHeader } from '@/features/events/components/WebhookTargetHeader'
import { WebhookTargetSummaryPanel } from '@/features/events/components/WebhookTargetSummaryPanel'
import { cn, formatClockTime } from '@/lib/utils'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'

const VALID_TABS = ['configure', 'deliveries', 'settings'] as const
type TargetDetailsTab = (typeof VALID_TABS)[number]

export function TargetDetailsPage() {
  const { targetId, tab } = useParams<{ targetId: string; tab?: string }>()
  const navigate = useRealmNavigate()
  const [deleteOpen, setDeleteOpen] = useState(false)

  const webhookId = targetId
  const activeTab = VALID_TABS.includes((tab ?? '') as TargetDetailsTab)
    ? (tab as TargetDetailsTab)
    : 'configure'

  const {
    data: webhookDetails,
    isLoading: webhookLoading,
    isError: webhookError,
    refetch: refetchWebhook,
  } = useWebhook(webhookId)
  const { enableWebhook, disableWebhook } = useWebhookMutations()
  const deleteWebhook = useDeleteWebhook()
  const replayDelivery = useReplayDelivery()

  const {
    data: webhookDeliveries,
    isLoading: webhookDeliveriesLoading,
    isFetching: webhookDeliveriesFetching,
    refetch: refetchWebhookDeliveries,
  } = useWebhookDeliveries(webhookId, { per_page: 50, page: 1 })

  useEffect(() => {
    if (!webhookId) return
    if (!tab || !VALID_TABS.includes(tab as TargetDetailsTab)) {
      navigate(`/events/webhooks/${webhookId}/configure`, { replace: true })
    }
  }, [navigate, tab, webhookId])

  const deliveries = useMemo<DeliveryInspectorItem[]>(() => {
    const logs = webhookDeliveries?.data ?? []
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
        latency: log.latency_ms ? `${log.latency_ms}ms` : '-',
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
  }, [webhookDeliveries?.data])

  if (!webhookId) return null

  if (webhookLoading) {
    return (
      <div className="bg-background flex h-full w-full flex-col overflow-hidden p-6">
        <BackToWebhooksLink />
        <div className="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-4">
          <Loader2 className="text-primary h-8 w-8 animate-spin" />
          <p>Loading webhook...</p>
        </div>
      </div>
    )
  }

  if (webhookError || !webhookDetails) {
    return (
      <div className="bg-background flex h-full w-full flex-col overflow-hidden p-6">
        <BackToWebhooksLink />
        <div className="text-destructive flex flex-1 flex-col items-center justify-center gap-2">
          <p>Webhook endpoint not found.</p>
          <Button variant="outline" onClick={() => navigate('/events')}>
            Back to Webhooks
          </Button>
        </div>
      </div>
    )
  }

  const endpoint = webhookDetails.endpoint
  const statusPending = enableWebhook.isPending || disableWebhook.isPending

  const handleTabChange = (newTab: string) => {
    navigate(`/events/webhooks/${webhookId}/${newTab}`)
  }

  const handleStatusChange = (checked: boolean) => {
    if (checked) {
      enableWebhook.mutate(endpoint.id)
    } else {
      disableWebhook.mutate({ endpointId: endpoint.id, reason: 'Disabled via UI' })
    }
  }

  const handleDelete = async () => {
    try {
      await deleteWebhook.mutateAsync(endpoint.id)
      setDeleteOpen(false)
      navigate('/events')
    } catch (err) {
      console.error('Failed to delete webhook', err)
    }
  }

  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden">
      <div className="shrink-0 px-6 pt-6">
        <BackToWebhooksLink />

        <WebhookTargetHeader endpoint={endpoint} />
      </div>

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 px-6 pt-2">
          <TabsList variant="line" className="gap-6 bg-transparent p-0">
            <TabsTrigger variant="line" value="configure" className="tab-trigger-styles">
              <SlidersHorizontal className="mr-2 h-4 w-4" /> Configure
            </TabsTrigger>
            <TabsTrigger variant="line" value="deliveries" className="tab-trigger-styles">
              <Activity className="mr-2 h-4 w-4" /> Deliveries
            </TabsTrigger>
            <TabsTrigger variant="line" value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="configure" className="mt-0 min-h-full w-full p-6">
            <WebhookTargetTabLayout details={webhookDetails}>
              <WebhookConfigureTab
                details={webhookDetails}
                onSaved={() => {
                  void refetchWebhook()
                  void refetchWebhookDeliveries()
                }}
              />
            </WebhookTargetTabLayout>
          </TabsContent>

          <TabsContent value="deliveries" className="mt-0 min-h-full w-full p-6">
            <DeliveriesInspector
              deliveries={deliveries}
              isLoading={webhookDeliveriesLoading}
              isRefreshing={webhookDeliveriesFetching}
              replayPending={replayDelivery.isPending}
              onRefresh={() => void refetchWebhookDeliveries()}
              onReplay={(deliveryId) => replayDelivery.mutate(deliveryId)}
            />
          </TabsContent>

          <TabsContent value="settings" className="mt-0 min-h-full w-full p-6">
            <WebhookTargetTabLayout details={webhookDetails}>
              <WebhookSettingsTab
                endpoint={endpoint}
                statusPending={statusPending}
                deletePending={deleteWebhook.isPending}
                onStatusChange={handleStatusChange}
                onDeleteClick={() => setDeleteOpen(true)}
              />
            </WebhookTargetTabLayout>
          </TabsContent>
        </div>
      </Tabs>

      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title="Delete webhook endpoint"
        desc="This will permanently delete the webhook and its delivery history."
        destructive
        overlayClassName="bg-background/80 dot-grid text-muted-foreground/20"
        isLoading={deleteWebhook.isPending}
        handleConfirm={handleDelete}
        confirmText={deleteWebhook.isPending ? 'Deleting...' : 'Delete'}
      />
    </div>
  )
}

export default TargetDetailsPage

function BackToWebhooksLink() {
  return (
    <div className="mb-2 shrink-0">
      <RealmLink
        to="/events"
        className={cn(
          buttonVariants({ variant: 'link', size: 'sm' }),
          'text-muted-foreground hover:text-foreground gap-2 pl-0',
        )}
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Webhooks
      </RealmLink>
    </div>
  )
}

function WebhookTargetTabLayout({
  details,
  children,
}: {
  details: WebhookEndpointDetails
  children: ReactNode
}) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <WebhookTargetSummaryPanel details={details} />
      </aside>
    </div>
  )
}

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
