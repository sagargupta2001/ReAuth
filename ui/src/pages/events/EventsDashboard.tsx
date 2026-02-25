import { useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'

import { Badge } from '@/components/badge'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from '@/components/breadcrumb'
import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Switch } from '@/components/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useWebhooks } from '@/features/events/api/useWebhooks'
import { WebhookEndpointForm } from '@/features/events/components/WebhookEndpointForm'
import { usePlugins } from '@/features/plugin/api/usePlugins'
import { usePluginMutations } from '@/features/plugin/api/usePluginMutations'
import { cn, formatRelativeTime } from '@/lib/utils'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { Main } from '@/widgets/Layout/Main'
import { Box, Plus, RefreshCcw } from 'lucide-react'

export function EventsDashboard() {
  const navigate = useRealmNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const { data: webhookData, isLoading: webhooksLoading, isError: webhooksError } = useWebhooks()
  const { data: pluginData, isLoading: pluginsLoading, isError: pluginsError } = usePlugins()
  const { enablePlugin, disablePlugin, refreshPlugins } = usePluginMutations()
  const activeTab = searchParams.get('tab') === 'plugins' ? 'plugins' : 'webhooks'

  useHashScrollHighlight()

  const webhookRows = useMemo(() => {
    const rows = webhookData ?? []
    return rows.map((details) => {
      const enabledSubscriptions = details.subscriptions.filter((sub) => sub.enabled)
      const subscriptionSummary = summarizeSubscriptions(
        enabledSubscriptions.map((sub) => sub.event_type),
      )
      const isFailing =
        details.endpoint.status !== 'active' || details.endpoint.consecutive_failures > 0

      return {
        id: details.endpoint.id,
        url: details.endpoint.url,
        method: details.endpoint.http_method || 'POST',
        status: isFailing ? 'failing' : 'active',
        subscriptions: subscriptionSummary,
        lastFired: formatRelativeTime(details.endpoint.updated_at),
      }
    })
  }, [webhookData])

  const pluginRows = useMemo(() => pluginData?.statuses ?? [], [pluginData])

  return (
    <Main className="flex flex-1 flex-col gap-6 p-12" fixed>
      <Breadcrumb>
        <BreadcrumbList>
          <BreadcrumbItem>
            <BreadcrumbPage>Event Routing</BreadcrumbPage>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">Event Routing</h1>
          <p className="text-muted-foreground">
            Manage how ReAuth events are delivered to external services and internal plugins.
          </p>
        </div>
      </div>

      <Tabs
        value={activeTab}
        onValueChange={(value) => {
          const params = new URLSearchParams(searchParams)
          params.set('tab', value)
          setSearchParams(params)
        }}
        className="flex flex-1 flex-col gap-6"
      >
        <TabsList className="w-fit rounded-full border bg-muted/40 p-1">
          <TabsTrigger value="webhooks" className="tab-trigger-styles">
            HTTP Webhooks
          </TabsTrigger>
          <TabsTrigger value="plugins" className="tab-trigger-styles">
            gRPC Plugins
          </TabsTrigger>
        </TabsList>

        <TabsContent value="webhooks" className="mt-0 flex flex-1 flex-col gap-4">
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

          <Card>
            <CardHeader className="border-b">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Configured Endpoints
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Method</TableHead>
                    <TableHead className="w-[40%]">Endpoint URL</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Subscriptions</TableHead>
                    <TableHead>Last Fired</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {webhooksLoading ? (
                    <TableRow>
                      <TableCell
                        colSpan={5}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        Loading webhook endpoints...
                      </TableCell>
                    </TableRow>
                  ) : webhooksError ? (
                    <TableRow>
                      <TableCell
                        colSpan={5}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        Failed to load webhook endpoints.
                      </TableCell>
                    </TableRow>
                  ) : webhookRows.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={5}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        No webhook endpoints configured yet.
                      </TableCell>
                    </TableRow>
                  ) : (
                    webhookRows.map((row) => (
                      <TableRow
                        key={row.id}
                        className="cursor-pointer transition hover:bg-muted/60"
                        onClick={() => navigate(`/events/webhooks/${row.id}`)}
                      >
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {row.method}
                        </TableCell>
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {row.url}
                        </TableCell>
                        <TableCell>
                          <Badge variant={row.status === 'active' ? 'success' : 'destructive'}>
                            {row.status === 'active' ? 'Active' : 'Failing'}
                          </Badge>
                        </TableCell>
                        <TableCell className="text-sm text-muted-foreground">
                          {row.subscriptions}
                        </TableCell>
                        <TableCell className="text-sm text-muted-foreground">
                          {row.lastFired}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="plugins" className="mt-0 flex flex-1 flex-col gap-4">
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

          <Card>
            <CardHeader className="border-b">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Plugin Registry
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Plugin Name</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Requested Events</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pluginsLoading ? (
                    <TableRow>
                      <TableCell
                        colSpan={4}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        Loading plugin registry...
                      </TableCell>
                    </TableRow>
                  ) : pluginsError ? (
                    <TableRow>
                      <TableCell
                        colSpan={4}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        Failed to load plugin registry.
                      </TableCell>
                    </TableRow>
                  ) : pluginRows.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={4}
                        className="py-6 text-center text-sm text-muted-foreground"
                      >
                        No plugins discovered.
                      </TableCell>
                    </TableRow>
                  ) : (
                    pluginRows.map((row) => {
                      const isActive = row.status === 'active'
                      const isFailed = typeof row.status === 'object' && 'failed' in row.status
                      const statusLabel = isFailed ? 'Failed' : isActive ? 'Enabled' : 'Disabled'

                      return (
                        <TableRow
                          key={row.manifest.id}
                          className="cursor-pointer transition hover:bg-muted/60"
                          onClick={() => navigate(`/events/plugins/${row.manifest.id}`)}
                        >
                          <TableCell className="flex items-center gap-2 font-medium">
                            <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-primary">
                              <Box className="h-4 w-4" />
                            </span>
                            {row.manifest.name}
                          </TableCell>
                          <TableCell className="text-sm text-muted-foreground">
                            {row.manifest.version}
                          </TableCell>
                          <TableCell onClick={(event) => event.stopPropagation()}>
                            <div className="flex items-center gap-2">
                              <Switch
                                checked={isActive}
                                onCheckedChange={(checked) =>
                                  checked
                                    ? enablePlugin.mutate(row.manifest.id)
                                    : disablePlugin.mutate(row.manifest.id)
                                }
                                disabled={enablePlugin.isPending || disablePlugin.isPending}
                              />
                              <span
                                className={cn(
                                  'text-xs font-medium',
                                  isFailed
                                    ? 'text-rose-500'
                                    : isActive
                                      ? 'text-emerald-500'
                                      : 'text-muted-foreground',
                                )}
                              >
                                {statusLabel}
                              </span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="flex flex-wrap gap-2">
                              {(row.manifest.events?.subscribes_to ?? []).length === 0 ? (
                                <Badge variant="outline" className="bg-muted/40">
                                  No events
                                </Badge>
                              ) : (
                                row.manifest.events?.subscribes_to?.map((event) => (
                                  <Badge key={event} variant="outline" className="bg-muted/40">
                                    {event}
                                  </Badge>
                                ))
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                      )
                    })
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </Main>
  )
}

export default EventsDashboard

function summarizeSubscriptions(subscriptions: string[]) {
  if (subscriptions.length === 0) return 'No events'
  if (subscriptions.length <= 2) return subscriptions.join(', ')
  return `${subscriptions[0]}, ${subscriptions[1]} + ${subscriptions.length - 2} more`
}
