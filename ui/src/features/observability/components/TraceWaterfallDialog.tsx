import { useMemo } from 'react'

import { Activity, ArrowRight, Clock3, Fingerprint, Route, Timer, UserRound, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { cn } from '@/lib/utils'

import { useTelemetryTraceSpans } from '../api/useTelemetryTraceSpans'
import type { TelemetryTrace } from '../model/types'

interface TraceWaterfallDialogProps {
  traceId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

interface SpanRow {
  span: TelemetryTrace
  depth: number
  offsetPct: number
  widthPct: number
}

function formatDuration(durationMs?: number | null) {
  if (durationMs === null || durationMs === undefined || !Number.isFinite(durationMs)) return '—'
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(2)}s`
  }
  return `${durationMs}ms`
}

function formatTime(timestamp?: string | null) {
  if (!timestamp) return '—'
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  return date.toLocaleString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

function traceStatusVariant(status?: number | null) {
  if (!status) return 'secondary'
  if (status >= 500) return 'destructive'
  if (status >= 400) return 'secondary'
  return 'default'
}

function computeDepth(span: TelemetryTrace, spanMap: Map<string, TelemetryTrace>) {
  let depth = 0
  let current = span
  const visited = new Set<string>()

  while (current.parent_id) {
    if (visited.has(current.parent_id)) break
    visited.add(current.parent_id)
    const parent = spanMap.get(current.parent_id)
    if (!parent) break
    depth += 1
    current = parent
  }

  return depth
}

function buildSpanRows(spans: TelemetryTrace[]): SpanRow[] {
  if (spans.length === 0) return []

  const spanMap = new Map(spans.map((span) => [span.span_id, span]))
  const parsed = spans
    .map((span) => {
      const start = new Date(span.start_time).getTime()
      return {
        span,
        start: Number.isNaN(start) ? null : start,
        end: Number.isNaN(start) ? null : start + span.duration_ms,
      }
    })
    .filter((item) => item.start !== null && item.end !== null)
    .sort((a, b) => (a.start ?? 0) - (b.start ?? 0))

  if (parsed.length === 0) return []

  const minStart = Math.min(...parsed.map((item) => item.start ?? 0))
  const maxEnd = Math.max(...parsed.map((item) => item.end ?? 0))
  const totalDuration = Math.max(maxEnd - minStart, 1)

  return parsed.map((item) => {
    const offsetPct =
      Math.min(Math.max(((item.start ?? minStart) - minStart) / totalDuration, 0), 1) * 100
    const availableWidthPct = Math.max(0, 100 - offsetPct)
    const rawWidthPct = (item.span.duration_ms / totalDuration) * 100
    const minVisibleWidthPct = Math.min(2, availableWidthPct)
    return {
      span: item.span,
      depth: computeDepth(item.span, spanMap),
      offsetPct,
      widthPct: Math.min(Math.max(rawWidthPct, minVisibleWidthPct), availableWidthPct),
    }
  })
}

function selectPrimaryTrace(spans: TelemetryTrace[]) {
  if (spans.length === 0) return null
  return (
    spans.find((span) => !span.parent_id && (span.method || span.route || span.path)) ??
    spans.find((span) => span.method || span.route || span.path) ??
    spans.find((span) => !span.parent_id) ??
    spans[0]
  )
}

function traceLabel(trace: TelemetryTrace | null, traceId: string | null) {
  if (!trace) return traceId ?? '—'
  return trace.route || trace.path || trace.name || trace.trace_id
}

export function TraceWaterfallDialog({ traceId, open, onOpenChange }: TraceWaterfallDialogProps) {
  const { t } = useTranslation('logs')
  const { data: spans = [], isError, isLoading } = useTelemetryTraceSpans(open ? traceId : null)
  const primaryTrace = useMemo(() => selectPrimaryTrace(spans), [spans])
  const spanRows = useMemo(() => buildSpanRows(spans), [spans])
  const title = traceLabel(primaryTrace, traceId)
  const traceDialogText = {
    eyebrow: t('TRACE_DIALOG.EYEBROW', { defaultValue: 'Trace detail' }),
    close: t('TRACE_DIALOG.CLOSE', { defaultValue: 'Close trace detail' }),
    duration: t('TRACE_DIALOG.DURATION', { defaultValue: 'Duration' }),
    started: t('TRACE_DIALOG.STARTED', { defaultValue: 'Started' }),
    route: t('TRACE_DIALOG.ROUTE', { defaultValue: 'Route' }),
    requestId: t('TRACE_DIALOG.REQUEST_ID', { defaultValue: 'Request ID' }),
    user: t('TRACE_DIALOG.USER', { defaultValue: 'User / Realm' }),
    spanCount: t('TRACE_DIALOG.SPAN_COUNT', {
      count: spans.length,
      defaultValue: '{{count}} spans',
    }),
    loading: t('TRACE_DIALOG.LOADING', { defaultValue: 'Loading trace spans...' }),
    error: t('TRACE_DIALOG.ERROR', { defaultValue: 'Unable to load trace spans.' }),
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="!top-4 flex h-[calc(100vh-2rem)] !max-w-[calc(100vw-2rem)] flex-col gap-0 overflow-hidden p-0 sm:rounded-xl">
        <div className="bg-surface-elevated border-b px-6 py-5">
          <div className="flex items-start justify-between gap-4">
            <DialogHeader className="min-w-0 flex-1 space-y-2">
              <div className="text-muted-foreground flex items-center gap-2 text-xs font-medium tracking-wide uppercase">
                <Activity className="h-4 w-4" />
                {traceDialogText.eyebrow}
              </div>
              <DialogTitle className="flex min-w-0 flex-wrap items-center gap-2 text-xl">
                {primaryTrace?.method ? (
                  <Badge variant="outline" className="shrink-0">
                    {primaryTrace.method}
                  </Badge>
                ) : null}
                <span className="truncate">{title}</span>
                {primaryTrace?.status ? (
                  <Badge variant={traceStatusVariant(primaryTrace.status)} className="shrink-0">
                    {primaryTrace.status}
                  </Badge>
                ) : null}
              </DialogTitle>
              <DialogDescription className="font-mono text-xs break-all">
                {traceId}
              </DialogDescription>
            </DialogHeader>
            <DialogClose asChild>
              <Button
                variant="ghost"
                size="icon"
                className="border-border/80 bg-background/80 text-foreground hover:bg-muted hover:text-foreground shrink-0 border shadow-sm"
                aria-label={traceDialogText.close}
              >
                <X className="h-5 w-5" />
              </Button>
            </DialogClose>
          </div>
        </div>

        <div className="bg-background grid min-h-0 flex-1 grid-rows-[auto_1fr] overflow-hidden">
          <div className="grid gap-3 border-b p-4 md:grid-cols-5">
            <TraceSummaryItem
              icon={Timer}
              label={traceDialogText.duration}
              value={formatDuration(primaryTrace?.duration_ms)}
            />
            <TraceSummaryItem
              icon={Clock3}
              label={traceDialogText.started}
              value={formatTime(primaryTrace?.start_time)}
            />
            <TraceSummaryItem
              icon={Route}
              label={traceDialogText.route}
              value={primaryTrace?.route || primaryTrace?.path || '—'}
            />
            <TraceSummaryItem
              icon={Fingerprint}
              label={traceDialogText.requestId}
              value={primaryTrace?.request_id ?? '—'}
              mono
            />
            <TraceSummaryItem
              icon={UserRound}
              label={traceDialogText.user}
              value={primaryTrace?.user_id ?? primaryTrace?.realm ?? '—'}
              mono
            />
          </div>

          <div className="min-h-0 overflow-auto">
            <div className="min-w-[780px] p-6">
              <div className="mb-4 flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-sm font-semibold">{t('TRACES_WATERFALL.HEADER')}</h3>
                  <p className="text-muted-foreground text-xs">{traceDialogText.spanCount}</p>
                </div>
              </div>

              {isLoading ? (
                <div className="bg-muted/20 text-muted-foreground rounded-xl border p-10 text-sm">
                  {traceDialogText.loading}
                </div>
              ) : isError ? (
                <div className="border-destructive/30 bg-destructive/5 text-destructive rounded-xl border p-10 text-sm">
                  {traceDialogText.error}
                </div>
              ) : spanRows.length === 0 ? (
                <div className="bg-muted/20 text-muted-foreground rounded-xl border p-10 text-sm">
                  {t('TRACES_WATERFALL.NO_SPANS')}
                </div>
              ) : (
                <div className="bg-surface-elevated overflow-hidden rounded-xl border p-4">
                  <div className="flex flex-col gap-3">
                    {spanRows.map((row) => (
                      <div key={row.span.span_id} className="grid grid-cols-[280px_1fr] gap-4">
                        <div
                          className="border-border/70 text-muted-foreground min-w-0 border-l pl-3 text-xs"
                          style={{ marginLeft: row.depth * 16 }}
                        >
                          <div className="text-foreground/80 truncate text-[11px] font-medium uppercase">
                            {row.span.name}
                          </div>
                          <div className="flex items-center gap-2 font-mono text-[10px]">
                            <span>{formatDuration(row.span.duration_ms)}</span>
                            {row.span.status ? (
                              <>
                                <ArrowRight className="h-3 w-3" />
                                <span>{row.span.status}</span>
                              </>
                            ) : null}
                          </div>
                        </div>
                        <div className="relative flex min-w-0 items-center overflow-hidden">
                          <div className="bg-muted/60 h-2 w-full rounded-full" />
                          <div
                            className={cn(
                              'absolute h-2 rounded-full',
                              row.span.status && row.span.status >= 500
                                ? 'bg-rose-500/80'
                                : row.span.status && row.span.status >= 400
                                  ? 'bg-amber-500/80'
                                  : 'bg-sky-500/80',
                            )}
                            style={{
                              left: `${row.offsetPct}%`,
                              width: `${row.widthPct}%`,
                            }}
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

type TraceSummaryItemProps = {
  icon: typeof Timer
  label: string
  value: string
  mono?: boolean
}

function TraceSummaryItem({ icon: Icon, label, value, mono }: TraceSummaryItemProps) {
  return (
    <div className="bg-surface-elevated min-w-0 rounded-xl border px-3 py-2">
      <div className="text-muted-foreground flex items-center gap-1.5 text-[11px] tracking-wide uppercase">
        <Icon className="h-3.5 w-3.5" />
        {label}
      </div>
      <div
        className={cn(
          'mt-1 truncate text-sm font-medium',
          mono && 'text-muted-foreground font-mono text-xs',
        )}
        title={value}
      >
        {value}
      </div>
    </div>
  )
}
