import { EventRoutingMetrics } from '@/features/events/components/EventRoutingMetrics'
import { WebhooksTable } from '@/features/events/components/WebhooksTable'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { Main } from '@/widgets/Layout/Main'

export function EventsDashboard() {
  useHashScrollHighlight()

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <EventRoutingMetrics />

      <div className="flex min-h-0 flex-1 flex-col gap-4">
        <WebhooksTable />
      </div>
    </Main>
  )
}

export default EventsDashboard
