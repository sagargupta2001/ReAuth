import { useSearchParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Card, CardContent } from '@/components/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { WebhookEndpointForm } from '@/features/events/components/WebhookEndpointForm'
import { WebhooksTable } from '@/features/events/components/WebhooksTable'
import { PluginsTable } from '@/features/events/components/PluginsTable'
import { EventRoutingMetrics } from '@/features/events/components/EventRoutingMetrics'
import { usePluginMutations } from '@/features/plugin/api/usePluginMutations'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { Main } from '@/widgets/Layout/Main'
import { Plus, RefreshCcw } from 'lucide-react'

export function EventsDashboard() {
  const [searchParams, setSearchParams] = useSearchParams()
  const { refreshPlugins } = usePluginMutations()
  const activeTab = searchParams.get('tab') === 'plugins' ? 'plugins' : 'webhooks'

  useHashScrollHighlight()

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Event Routing</h1>
          <p className="text-muted-foreground">
            Manage how ReAuth events are delivered to external services and internal plugins.
          </p>
        </div>
      </div>

      <EventRoutingMetrics />

      <Tabs
        value={activeTab}
        onValueChange={(value) => {
          const params = new URLSearchParams(searchParams)
          params.set('tab', value)
          setSearchParams(params)
        }}
        className="flex min-h-0 flex-1 flex-col gap-6"
      >
        <TabsList className="w-fit rounded-full border bg-muted/40 p-1">
          <TabsTrigger value="webhooks" className="tab-trigger-styles">
            HTTP Webhooks
          </TabsTrigger>
          <TabsTrigger value="plugins" className="tab-trigger-styles">
            gRPC Plugins
          </TabsTrigger>
        </TabsList>

        <TabsContent value="webhooks" className="mt-0 flex min-h-0 flex-1 flex-col gap-4">
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
        </TabsContent>

        <TabsContent value="plugins" className="mt-0 flex min-h-0 flex-1 flex-col gap-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h2 className="text-xl font-semibold">gRPC Plugins</h2>
              <p className="text-sm text-muted-foreground">
                Manage internal plugins and their requested event streams.
              </p>
            </div>
            <Button variant="secondary" onClick={refreshPlugins}>
              <RefreshCcw className="h-4 w-4" />
              Refresh Registry
            </Button>
          </div>

          <Card className="flex min-h-0 flex-1 flex-col">
            <CardContent className="flex min-h-0 flex-1 flex-col p-4">
              <PluginsTable />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </Main>
  )
}

export default EventsDashboard
