import { EventRoutingMetrics } from '@/features/events/components/EventRoutingMetrics'
import { WebhooksTable } from '@/features/events/components/WebhooksTable'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { Main } from '@/widgets/Layout/Main'

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
      </div>

      <EventRoutingMetrics />

      <div className="flex min-h-0 flex-1 flex-col gap-4">
        <WebhooksTable />
      </div>
    </Main>
  )
}

export default EventsDashboard
