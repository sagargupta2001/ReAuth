import { Button } from '@/components/button'
import { Card, CardContent } from '@/components/card'
import { WebhookEndpointForm } from '@/features/events/components/WebhookEndpointForm'
import { WebhooksTable } from '@/features/events/components/WebhooksTable'
import { EventRoutingMetrics } from '@/features/events/components/EventRoutingMetrics'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { Main } from '@/widgets/Layout/Main'
import { Plus } from 'lucide-react'

export function EventsDashboard() {
  useHashScrollHighlight()

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Event Routing</h1>
          <p className="text-muted-foreground">
            Manage how ReAuth events are delivered to external webhook endpoints.
          </p>
        </div>
      </div>

      <EventRoutingMetrics />

      <div className="flex min-h-0 flex-1 flex-col gap-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-xl font-semibold">HTTP Webhooks</h2>
            <p className="text-sm text-muted-foreground">
              Route selected events to third-party services with signed payloads.
            </p>
          </div>
          <WebhookEndpointForm
            trigger={
              <Button id="create-webhook">
                <Plus className="h-4 w-4" />
                Add Webhook
              </Button>
            }
          />
        </div>

        <Card className="flex min-h-0 flex-1 flex-col">
          <CardContent className="flex min-h-0 flex-1 flex-col p-4">
            <WebhooksTable />
          </CardContent>
        </Card>
      </div>
    </Main>
  )
}

export default EventsDashboard
