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
          <h1 className="text-3xl font-semibold tracking-tight">Webhooks</h1>
          <p className="text-muted-foreground">
            Manage how ReAuth events are delivered to external HTTP endpoints.
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

      <EventRoutingMetrics />

      <div className="flex min-h-0 flex-1 flex-col gap-4">
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
