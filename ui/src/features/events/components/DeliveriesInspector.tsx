import { useEffect, useMemo, useState } from 'react'

import {
  CheckCircle2,
  CirclePlay,
  Clock3,
  Copy,
  FileText,
  RefreshCcw,
  Search,
  XCircle,
} from 'lucide-react'

import { Badge, type BadgeProps } from '@/components/badge'
import { Button } from '@/components/button'
import { Card } from '@/components/card'
import { Input } from '@/components/input'
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
  onRefresh?: () => void
  isRefreshing?: boolean
}

type DeliveryStatusFilter = 'all' | DeliveryInspectorItem['status']

const deliveryStatusBadgeVariants: Record<DeliveryInspectorItem['status'], BadgeProps['variant']> =
  {
    success: 'successMuted',
    failed: 'dangerMuted',
  }

const deliveryStatusDotClasses: Record<DeliveryInspectorItem['status'], string> = {
  success: 'bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,0.35)]',
  failed: 'bg-rose-400 shadow-[0_0_12px_rgba(251,113,133,0.35)]',
}

const deliveryStatusIcons = {
  success: CheckCircle2,
  failed: XCircle,
}

const statusFilters: Array<{ value: DeliveryStatusFilter; label: string }> = [
  { value: 'all', label: 'All' },
  { value: 'success', label: 'Success' },
  { value: 'failed', label: 'Failed' },
]

export function DeliveriesInspector({
  deliveries,
  isLoading = false,
  onReplay,
  replayPending = false,
  onRefresh,
  isRefreshing = false,
}: DeliveriesInspectorProps) {
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState<DeliveryStatusFilter>('all')
  const [selectedId, setSelectedId] = useState<string | undefined>(deliveries[0]?.id)

  const filteredDeliveries = useMemo(() => {
    const query = searchTerm.trim().toLowerCase()

    return deliveries.filter((delivery) => {
      const matchesStatus = statusFilter === 'all' || delivery.status === statusFilter
      if (!matchesStatus) return false

      if (!query) return true

      return [delivery.eventType, delivery.id, delivery.response.status, delivery.timestamp]
        .filter(Boolean)
        .some((value) => value.toLowerCase().includes(query))
    })
  }, [deliveries, searchTerm, statusFilter])

  useEffect(() => {
    if (!filteredDeliveries.length) {
      setSelectedId(undefined)
      return
    }

    const stillVisible = filteredDeliveries.some((delivery) => delivery.id === selectedId)
    if (!stillVisible) setSelectedId(filteredDeliveries[0]?.id)
  }, [filteredDeliveries, selectedId])

  const selected = useMemo(
    () => deliveries.find((delivery) => delivery.id === selectedId),
    [deliveries, selectedId],
  )

  const selectedStatusIcon = selected ? deliveryStatusIcons[selected.status] : FileText

  return (
    <Card className="grid min-h-[640px] overflow-hidden border lg:grid-cols-[minmax(260px,320px)_1fr]">
      <aside className="bg-muted/15 border-b lg:border-r lg:border-b-0">
        <div className="space-y-3 border-b p-4">
          <div className="flex items-center justify-between gap-3">
            <p className="text-sm font-semibold">Delivery History</p>
            {onRefresh ? (
              <Button
                variant="outline"
                size="icon"
                onClick={onRefresh}
                disabled={isRefreshing || isLoading}
                className="h-8 w-8 shrink-0 bg-transparent border-none"
                aria-label="Refresh deliveries"
              >
                <RefreshCcw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
              </Button>
            ) : null}
          </div>

          <div className="relative">
            <Search className="text-muted-foreground/60 absolute top-2.5 left-3 h-4 w-4" />
            <Input
              value={searchTerm}
              onChange={(event) => setSearchTerm(event.target.value)}
              placeholder="Search deliveries..."
              className="h-9 border pl-9 text-sm"
            />
          </div>

          <div className="flex flex-wrap gap-2">
            {statusFilters.map((filter) => {
              const isActive = statusFilter === filter.value
              const count =
                filter.value === 'all'
                  ? deliveries.length
                  : deliveries.filter((delivery) => delivery.status === filter.value).length

              return (
                <button
                  key={filter.value}
                  type="button"
                  onClick={() => setStatusFilter(filter.value)}
                  className="focus-visible:ring-ring rounded-full focus-visible:ring-2 focus-visible:outline-none"
                >
                  <Badge
                    variant={
                      filter.value === 'success'
                        ? 'successMuted'
                        : filter.value === 'failed'
                          ? 'dangerMuted'
                          : 'neutralMuted'
                    }
                    className={cn(!isActive && 'opacity-60')}
                  >
                    {filter.label}
                    <span className="ml-1 text-[10px] opacity-70">{count}</span>
                  </Badge>
                </button>
              )
            })}
          </div>
        </div>

        <ScrollArea className="h-[520px]">
          <div className="flex flex-col gap-2 p-3">
            {isLoading ? (
              <EmptyDeliveryState label="Loading deliveries..." />
            ) : deliveries.length === 0 ? (
              <EmptyDeliveryState label="No deliveries yet." />
            ) : filteredDeliveries.length === 0 ? (
              <EmptyDeliveryState label="No deliveries match this view." />
            ) : (
              filteredDeliveries.map((delivery) => (
                <DeliveryHistoryRow
                  key={delivery.id}
                  delivery={delivery}
                  isSelected={selectedId === delivery.id}
                  onSelect={() => setSelectedId(delivery.id)}
                />
              ))
            )}
          </div>
        </ScrollArea>
      </aside>

      <section className="flex min-w-0 flex-col">
        <div className="flex flex-wrap items-start justify-between gap-3 border-b p-4">
          <div className="min-w-0">
            <p className="text-sm font-semibold">Event Detail</p>
            <div className="text-muted-foreground mt-1 flex min-w-0 items-center gap-1.5 text-xs">
              <span className="shrink-0">Delivery ID:</span>
              <span className="truncate font-mono">{selected?.id ?? 'Select a delivery'}</span>
              {selected ? (
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-muted-foreground h-5 w-5 shrink-0"
                  onClick={() => void navigator.clipboard?.writeText(selected.id)}
                  aria-label="Copy delivery ID"
                >
                  <Copy className="h-3.5 w-3.5" />
                </Button>
              ) : null}
            </div>
          </div>

          <Button
            variant="secondary"
            disabled={!selected || !onReplay || replayPending}
            onClick={() => selected && onReplay?.(selected.id)}
            className="shrink-0"
          >
            <CirclePlay className="h-4 w-4 text-emerald-400" />
            Replay Event
          </Button>
        </div>

        {!selected ? (
          <div className="flex flex-1 items-center justify-center p-8">
            <EmptyDeliveryState label="Select a delivery to inspect request and response data." />
          </div>
        ) : (
          <div className="flex flex-1 flex-col gap-4 p-4">
            <div className="bg-muted/20 grid gap-3 rounded-lg border p-3 md:grid-cols-3">
              <DeliveryMetaItem
                icon={selectedStatusIcon}
                label="HTTP Status"
                value={selected.response.status}
                variant={deliveryStatusBadgeVariants[selected.status]}
              />
              <DeliveryMetaItem
                icon={Clock3}
                label="Latency"
                value={selected.latency}
                variant="warningMuted"
              />
              <DeliveryMetaItem
                icon={FileText}
                label="Event Type"
                value={selected.eventType}
                variant="neutralMuted"
              />
            </div>

            <Tabs defaultValue="response" className="min-h-0 flex-1">
              <div className="overflow-hidden rounded-lg border">
                <TabsList variant="line" className="bg-muted/20 px-3">
                  <TabsTrigger value="response" variant="line" className="text-xs">
                    Response Body
                  </TabsTrigger>
                  <TabsTrigger value="headers" variant="line" className="text-xs">
                    Headers
                  </TabsTrigger>
                  <TabsTrigger value="request" variant="line" className="text-xs">
                    Request
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="response" className="m-0">
                  <div className="space-y-3 p-3">
                    {selected.status === 'failed' &&
                    (selected.failureReason || selected.errorChain?.length) ? (
                      <FailureReason delivery={selected} />
                    ) : null}
                    <HighlightedJsonBlock value={selected.response.body} />
                  </div>
                </TabsContent>

                <TabsContent value="headers" className="m-0">
                  <div className="space-y-3 p-3">
                    <HeaderRow
                      label="Reauth-Signature"
                      value={selected.signature ?? 'Not stored'}
                    />
                    <HeaderRow label="Delivery ID" value={selected.id} />
                    <HeaderRow label="Timestamp" value={selected.timestamp} />
                    <HeaderRow label="Status" value={selected.response.status} />
                  </div>
                </TabsContent>

                <TabsContent value="request" className="m-0">
                  <div className="p-3">
                    <HighlightedJsonBlock value={selected.payload} />
                  </div>
                </TabsContent>
              </div>
            </Tabs>
          </div>
        )}
      </section>
    </Card>
  )
}

function DeliveryHistoryRow({
  delivery,
  isSelected,
  onSelect,
}: {
  delivery: DeliveryInspectorItem
  isSelected: boolean
  onSelect: () => void
}) {
  return (
    <button
      onClick={onSelect}
      className={cn(
        'flex w-full items-center justify-between gap-3 rounded-lg border px-3 py-3 text-left transition',
        isSelected
          ? 'border-primary/30 bg-primary/10 shadow-sm'
          : 'border-border/70 bg-background/40 hover:border-primary/20 hover:bg-muted/50',
      )}
    >
      <div className="flex min-w-0 items-center gap-3">
        <span
          aria-hidden="true"
          className={cn(
            'h-2.5 w-2.5 shrink-0 rounded-full',
            deliveryStatusDotClasses[delivery.status],
          )}
        />
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold">{delivery.eventType}</p>
          <p className="text-muted-foreground truncate text-xs">{delivery.response.status}</p>
        </div>
      </div>
      <span className="text-muted-foreground shrink-0 text-xs">{delivery.timestamp}</span>
    </button>
  )
}

function DeliveryMetaItem({
  icon: Icon,
  label,
  value,
  variant,
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string
  variant: BadgeProps['variant']
}) {
  return (
    <div className="flex min-w-0 items-center gap-2">
      <Icon className="text-muted-foreground h-4 w-4 shrink-0" />
      <span className="text-muted-foreground text-xs">{label}:</span>
      <Badge variant={variant} className="min-w-0 truncate">
        {value}
      </Badge>
    </div>
  )
}

function HeaderRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-muted/20 grid gap-2 rounded-md border p-3 md:grid-cols-[160px_1fr]">
      <span className="text-muted-foreground text-xs font-medium">{label}</span>
      <span className="font-mono text-xs wrap-break-word">{value}</span>
    </div>
  )
}

function FailureReason({ delivery }: { delivery: DeliveryInspectorItem }) {
  return (
    <section className="rounded-md border border-rose-500/20 bg-rose-950/25 p-3">
      <Badge variant="dangerMuted">Failure reason</Badge>
      <p className="mt-2 font-mono text-xs wrap-break-word text-rose-200">
        {delivery.failureReason ?? 'Unknown failure'}
      </p>
      {delivery.errorChain && delivery.errorChain.length > 0 ? (
        <div className="mt-3">
          <Badge variant="dangerMuted">Error chain</Badge>
          <div className="mt-2 flex flex-col gap-1 text-xs text-rose-200">
            {delivery.errorChain.map((item, index) => (
              <span key={`${delivery.id}-chain-${index}`} className="font-mono wrap-break-word">
                {item}
              </span>
            ))}
          </div>
        </div>
      ) : null}
    </section>
  )
}

function EmptyDeliveryState({ label }: { label: string }) {
  return (
    <div className="text-muted-foreground rounded-md border border-dashed px-3 py-6 text-center text-xs">
      {label}
    </div>
  )
}

function HighlightedJsonBlock({ value }: { value: unknown }) {
  const json = JSON.stringify(value ?? {}, null, 2) ?? '{}'
  const lines = json.split('\n')

  return (
    <ScrollArea className="h-[430px] rounded-md border bg-slate-950/90">
      <pre className="min-w-full p-4 font-mono text-xs text-slate-100">
        {lines.map((line, index) => (
          <div key={`${index}-${line}`} className="grid grid-cols-[2.5rem_1fr] leading-5">
            <span className="pr-4 text-right text-slate-500 select-none">{index + 1}</span>
            {renderJsonLine(line)}
          </div>
        ))}
      </pre>
    </ScrollArea>
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
