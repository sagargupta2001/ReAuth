import { useEffect, useMemo, useState } from 'react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card } from '@/components/card'
import { ScrollArea } from '@/components/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { cn } from '@/lib/utils'

export interface DeliveryInspectorItem {
  id: string
  eventType: string
  status: 'success' | 'failed'
  timestamp: string
  latency: string
  signature?: string | null
  payload: unknown
  failureReason?: string | null
  errorChain?: string[] | null
  response: {
    status: string
    body: unknown
  }
}

interface DeliveriesInspectorProps {
  deliveries: DeliveryInspectorItem[]
  isLoading?: boolean
  onReplay?: (deliveryId: string) => void
  replayPending?: boolean
}

export function DeliveriesInspector({
  deliveries,
  isLoading = false,
  onReplay,
  replayPending = false,
}: DeliveriesInspectorProps) {
  const [selectedId, setSelectedId] = useState<string | undefined>(deliveries[0]?.id)

  useEffect(() => {
    if (!deliveries.length) {
      setSelectedId(undefined)
      return
    }

    const stillExists = deliveries.some((delivery) => delivery.id === selectedId)
    if (!stillExists) setSelectedId(deliveries[0]?.id)
  }, [deliveries, selectedId])

  const selected = useMemo(
    () => deliveries.find((delivery) => delivery.id === selectedId),
    [deliveries, selectedId],
  )

  return (
    <div className="grid gap-4 lg:grid-cols-[minmax(260px,30%)_1fr]">
      <Card className="border">
        <div className="border-b px-4 py-3">
          <p className="text-sm font-semibold">Delivery History</p>
          <p className="text-xs text-muted-foreground">Recent attempts and outcomes</p>
        </div>
        <ScrollArea className="h-[520px]">
          <div className="flex flex-col gap-2 p-3">
            {isLoading ? (
              <div className="rounded-md border border-dashed px-3 py-6 text-center text-xs text-muted-foreground">
                Loading deliveries...
              </div>
            ) : deliveries.length === 0 ? (
              <div className="rounded-md border border-dashed px-3 py-6 text-center text-xs text-muted-foreground">
                No deliveries yet.
              </div>
            ) : (
              deliveries.map((delivery) => (
                <button
                  key={delivery.id}
                  onClick={() => setSelectedId(delivery.id)}
                  className={cn(
                    'flex w-full items-center justify-between gap-3 rounded-md border px-3 py-3 text-left transition',
                    selectedId === delivery.id
                      ? 'border-primary/30 bg-primary/10'
                      : 'border-transparent hover:bg-muted/50',
                  )}
                >
                  <div className="flex items-center gap-3">
                    <span
                      className={cn(
                        'h-2.5 w-2.5 rounded-full',
                        delivery.status === 'success' ? 'bg-emerald-500' : 'bg-rose-500',
                      )}
                    />
                    <div>
                      <p className="text-sm font-semibold">{delivery.eventType}</p>
                      <p className="text-xs text-muted-foreground">{delivery.timestamp}</p>
                    </div>
                  </div>
                  <Badge variant={delivery.status === 'success' ? 'success' : 'destructive'}>
                    {delivery.status === 'success' ? 'Success' : 'Failed'}
                  </Badge>
                </button>
              ))
            )}
          </div>
        </ScrollArea>
      </Card>

      <Card className="border">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3">
          <div>
            <p className="text-sm font-semibold">Delivery ID</p>
            <p className="text-xs text-muted-foreground">
              {selected?.id ?? 'Select a delivery'}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Badge variant="outline" className="bg-muted/40">
              Latency {selected?.latency ?? '—'}
            </Badge>
            <Button
              variant="secondary"
              disabled={!selected || !onReplay || replayPending}
              onClick={() => selected && onReplay?.(selected.id)}
            >
              Replay Event
            </Button>
          </div>
        </div>

        <div className="p-4">
          {!selected ? (
            <div className="rounded-md border border-dashed px-4 py-10 text-center text-sm text-muted-foreground">
              Select a delivery to inspect the request and response payloads.
            </div>
          ) : (
          <Tabs defaultValue="request" className="flex flex-col gap-4">
            <TabsList className="w-fit rounded-full border bg-muted/40 p-1">
              <TabsTrigger value="request" className="tab-trigger-styles">
                Request
              </TabsTrigger>
              <TabsTrigger value="response" className="tab-trigger-styles">
                Response
              </TabsTrigger>
            </TabsList>

            <TabsContent value="request" className="mt-0 space-y-4">
              <div className="rounded-md border bg-muted/40 p-3">
                <p className="text-xs font-semibold text-muted-foreground">Headers</p>
                <div className="mt-2 flex flex-wrap items-center gap-2 text-xs">
                  <Badge variant="outline">Reauth-Signature</Badge>
                  <span className="font-mono text-xs text-foreground">
                    {selected?.signature ?? 'Not stored'}
                  </span>
                </div>
              </div>

              <HighlightedJsonBlock value={selected?.payload} />
            </TabsContent>

            <TabsContent value="response" className="mt-0 space-y-4">
              <div className="flex items-center gap-2">
                <Badge
                  variant={selected?.status === 'success' ? 'success' : 'destructive'}
                  className="uppercase"
                >
                  {selected?.response.status}
                </Badge>
              </div>
              {selected?.status === 'failed' && (selected.failureReason || selected.errorChain?.length) ? (
                <div className="rounded-md border border-rose-200/70 bg-rose-50/60 p-3">
                  <p className="text-xs font-semibold text-rose-800">Failure reason</p>
                  <p className="mt-1 break-words font-mono text-xs text-rose-900">
                    {selected.failureReason ?? 'Unknown failure'}
                  </p>
                  {selected.errorChain && selected.errorChain.length > 0 ? (
                    <div className="mt-3">
                      <p className="text-xs font-semibold text-rose-800">Error chain</p>
                      <div className="mt-2 flex flex-col gap-1 text-xs text-rose-900">
                        {selected.errorChain.map((item, index) => (
                          <span key={`${selected.id}-chain-${index}`} className="break-words font-mono">
                            {item}
                          </span>
                        ))}
                      </div>
                    </div>
                  ) : null}
                </div>
              ) : null}
              <HighlightedJsonBlock value={selected?.response.body} />
            </TabsContent>
          </Tabs>
          )}
        </div>
      </Card>
    </div>
  )
}

function HighlightedJsonBlock({ value }: { value: unknown }) {
  const json = JSON.stringify(value ?? {}, null, 2) ?? '{}'
  const lines = json.split('\n')

  return (
    <div className="rounded-md border bg-slate-950/90 p-4 font-mono text-xs text-slate-100">
      <pre className="whitespace-pre-wrap">
        {lines.map((line, index) => (
          <div key={`${index}-${line}`} className="leading-5">
            {renderJsonLine(line)}
          </div>
        ))}
      </pre>
    </div>
  )
}

function renderJsonLine(line: string) {
  const match = line.match(/^(\s*)"([^"]+)":\s*(.*)$/)
  if (!match) {
    return <span className="text-slate-200">{line}</span>
  }

  const [, indent, key, rest] = match
  let valuePart = rest
  let trailing = ''
  if (valuePart.endsWith(',')) {
    trailing = ','
    valuePart = valuePart.slice(0, -1)
  }

  const valueNode = renderJsonValue(valuePart)

  return (
    <span>
      <span className="text-slate-400">{indent}</span>
      <span className="text-sky-300">"{key}"</span>
      <span className="text-slate-200">: </span>
      {valueNode}
      <span className="text-slate-200">{trailing}</span>
    </span>
  )
}

function renderJsonValue(value: string) {
  const trimmed = value.trim()
  if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return <span className="text-emerald-300">{trimmed}</span>
  }
  if (trimmed === 'true' || trimmed === 'false') {
    return <span className="text-purple-300">{trimmed}</span>
  }
  if (trimmed === 'null') {
    return <span className="text-slate-400">{trimmed}</span>
  }
  if (!Number.isNaN(Number(trimmed))) {
    return <span className="text-amber-300">{trimmed}</span>
  }
  return <span className="text-slate-200">{trimmed}</span>
}
