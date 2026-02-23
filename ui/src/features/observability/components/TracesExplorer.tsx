import { useEffect, useMemo, useState } from 'react'

import { Activity, ArrowRight, ChevronLeft, ChevronRight, Timer } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Command, CommandInput } from '@/components/command'
import { ScrollArea } from '@/components/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { cn } from '@/lib/utils'
import { enumParam, numberParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'

import { useTelemetryTraceSpans } from '../api/useTelemetryTraceSpans'
import { useTelemetryTraces } from '../api/useTelemetryTraces'
import type { ResolvedTimeRange } from '../lib/timeRange'
import type { TelemetryTrace } from '../model/types'

interface TracesExplorerProps {
  timeRange: ResolvedTimeRange
  selectedTraceId: string | null
  onSelectTrace: (traceId: string) => void
}

interface SpanRow {
  span: TelemetryTrace
  depth: number
  offsetPct: number
  widthPct: number
}

const SORT_FIELDS = ['start_time', 'duration_ms', 'status'] as const
const SORT_DIRS = ['desc', 'asc'] as const
const PER_PAGE_OPTIONS = [50, 100, 200]

type SortField = (typeof SORT_FIELDS)[number]

type SortDir = (typeof SORT_DIRS)[number]

type TraceSortOption = {
  value: string
  label: string
  sort_by: SortField
  sort_dir: SortDir
}

function formatDuration(durationMs: number) {
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(2)}s`
  }
  return `${durationMs}ms`
}

function formatTime(timestamp: string) {
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
    const offset = ((item.start ?? minStart) - minStart) / totalDuration
    const width = item.span.duration_ms / totalDuration
    return {
      span: item.span,
      depth: computeDepth(item.span, spanMap),
      offsetPct: offset * 100,
      widthPct: Math.max(width * 100, 2),
    }
  })
}

export function TracesExplorer({ timeRange, selectedTraceId, onSelectTrace }: TracesExplorerProps) {
  const { t } = useTranslation('logs')
  const sortOptions: TraceSortOption[] = useMemo(
    () => [
      {
        value: 'start_time:desc',
        label: t('TRACES_LIST.SORT_NEWEST'),
        sort_by: 'start_time',
        sort_dir: 'desc',
      },
      {
        value: 'start_time:asc',
        label: t('TRACES_LIST.SORT_OLDEST'),
        sort_by: 'start_time',
        sort_dir: 'asc',
      },
      {
        value: 'duration_ms:desc',
        label: t('TRACES_LIST.SORT_SLOWEST'),
        sort_by: 'duration_ms',
        sort_dir: 'desc',
      },
      {
        value: 'duration_ms:asc',
        label: t('TRACES_LIST.SORT_FASTEST'),
        sort_by: 'duration_ms',
        sort_dir: 'asc',
      },
    ],
    [t],
  )

  const [urlState, setUrlState] = useUrlState<{
    trace_page: number
    trace_per_page: number
    trace_q: string
    trace_sort_by: SortField
    trace_sort_dir: SortDir
  }>({
    trace_page: numberParam(1),
    trace_per_page: numberParam(100),
    trace_q: stringParam(''),
    trace_sort_by: enumParam(SORT_FIELDS, 'start_time'),
    trace_sort_dir: enumParam(SORT_DIRS, 'desc'),
  })

  const [searchInput, setSearchInput] = useState(urlState.trace_q)

  useEffect(() => {
    setSearchInput(urlState.trace_q)
  }, [urlState.trace_q])

  useEffect(() => {
    const handle = window.setTimeout(() => {
      const trimmed = searchInput.trim()
      if (trimmed !== urlState.trace_q) {
        setUrlState({ trace_q: trimmed, trace_page: 1 })
      }
    }, 350)
    return () => window.clearTimeout(handle)
  }, [searchInput, setUrlState, urlState.trace_q])

  const start = timeRange.start ? timeRange.start.toISOString() : undefined
  const end = timeRange.end ? timeRange.end.toISOString() : undefined

  const { data, isLoading } = useTelemetryTraces({
    search: urlState.trace_q || undefined,
    start,
    end,
    page: urlState.trace_page,
    per_page: urlState.trace_per_page,
    sort_by: urlState.trace_sort_by,
    sort_dir: urlState.trace_sort_dir,
  })

  const traces = data?.data ?? []
  const meta = data?.meta

  const requestTraces = useMemo(
    () => traces.filter((trace) => trace.method || trace.route || trace.path),
    [traces],
  )

  const selectedTrace = useMemo(() => {
    if (!requestTraces.length) return null
    return (
      requestTraces.find((trace) => trace.trace_id === selectedTraceId) ?? requestTraces[0]
    )
  }, [requestTraces, selectedTraceId])

  useEffect(() => {
    if (selectedTrace && selectedTrace.trace_id !== selectedTraceId) {
      onSelectTrace(selectedTrace.trace_id)
    }
  }, [onSelectTrace, selectedTrace, selectedTraceId])

  const { data: spans = [] } = useTelemetryTraceSpans(selectedTrace?.trace_id)
  const spanRows = useMemo(() => buildSpanRows(spans), [spans])

  const totalResults = meta?.total ?? requestTraces.length
  const totalPages = meta?.total_pages && meta.total_pages > 0 ? meta.total_pages : 1
  const isFirstPage = urlState.trace_page <= 1
  const isLastPage = totalPages > 0 ? urlState.trace_page >= totalPages : true
  const sortValue = `${urlState.trace_sort_by}:${urlState.trace_sort_dir}`

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-center gap-3">
        <Command className="flex-1 border bg-background/60">
          <CommandInput
            value={searchInput}
            onValueChange={setSearchInput}
            placeholder={t('TRACES_LIST.SEARCH_PLACEHOLDER')}
            className="h-10 text-sm"
          />
        </Command>
        <Select
          value={sortValue}
          onValueChange={(value) => {
            const option = sortOptions.find((item) => item.value === value)
            if (!option) return
            setUrlState({
              trace_sort_by: option.sort_by,
              trace_sort_dir: option.sort_dir,
              trace_page: 1,
            })
          }}
        >
          <SelectTrigger className="w-[200px]">
            <SelectValue placeholder={t('TRACES_LIST.SORT_LABEL')} />
          </SelectTrigger>
          <SelectContent>
            {sortOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="ml-auto flex items-center gap-2 text-xs text-muted-foreground">
          <Activity className="h-3.5 w-3.5" />
          {t('TRACES_LIST.RESULT_COUNT', { count: totalResults })}
        </div>
      </div>

      <div className="grid h-full grid-cols-1 gap-4 lg:grid-cols-[320px_1fr]">
        <div className="flex h-full flex-col rounded-xl border bg-background/40">
          <div className="border-b px-4 py-3">
            <div className="flex items-center gap-2 text-sm font-semibold">
              <Activity className="h-4 w-4 text-muted-foreground" />
              {t('TRACES_LIST.TITLE')}
            </div>
            <p className="text-xs text-muted-foreground">{t('TRACES_LIST.SUBTITLE')}</p>
          </div>
          <ScrollArea className="flex-1">
            <div className="flex flex-col divide-y">
              {isLoading && requestTraces.length === 0 ? (
                <div className="p-4 text-sm text-muted-foreground">{t('TRACES_LIST.LOADING')}</div>
              ) : requestTraces.length === 0 ? (
                <div className="p-4 text-sm text-muted-foreground">{t('TRACES_LIST.EMPTY')}</div>
              ) : (
                requestTraces.map((trace) => {
                  const isSelected = trace.trace_id === selectedTrace?.trace_id
                  const label = trace.route || trace.path || trace.name
                  return (
                    <button
                      key={trace.trace_id}
                      className={cn(
                        'flex w-full flex-col gap-2 px-4 py-3 text-left transition-colors',
                        isSelected ? 'bg-muted/50' : 'hover:bg-muted/30',
                      )}
                      onClick={() => onSelectTrace(trace.trace_id)}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <div className="flex items-center gap-2">
                          <Badge variant="outline" className="text-[10px]">
                            {trace.method ?? 'HTTP'}
                          </Badge>
                          <span className="truncate text-sm font-medium">{label}</span>
                        </div>
                        <Badge variant={traceStatusVariant(trace.status)} className="text-[10px]">
                          {trace.status ?? '—'}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between text-xs text-muted-foreground">
                        <span>{formatTime(trace.start_time)}</span>
                        <span className="font-mono">{formatDuration(trace.duration_ms)}</span>
                      </div>
                    </button>
                  )
                })
              )}
            </div>
          </ScrollArea>
          <div className="border-t px-4 py-3 text-xs text-muted-foreground">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <span>
                {t('TRACES_LIST.PAGE_STATUS', {
                  page: urlState.trace_page,
                  total: totalPages,
                })}
              </span>
              <div className="flex items-center gap-2">
                <span>{t('TRACES_LIST.ROWS_PER_PAGE')}</span>
                <Select
                  value={String(urlState.trace_per_page)}
                  onValueChange={(value) =>
                    setUrlState({ trace_per_page: Number(value), trace_page: 1 })
                  }
                >
                  <SelectTrigger className="h-8 w-[90px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {PER_PAGE_OPTIONS.map((option) => (
                      <SelectItem key={option} value={String(option)}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    setUrlState({ trace_page: Math.max(1, urlState.trace_page - 1) })
                  }
                  disabled={isFirstPage}
                >
                  <ChevronLeft className="h-4 w-4" />
                  {t('TRACES_LIST.PREV')}
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setUrlState({ trace_page: urlState.trace_page + 1 })}
                  disabled={isLastPage}
                >
                  {t('TRACES_LIST.NEXT')}
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        </div>

        <div className="flex h-full flex-col rounded-xl border bg-background/40">
          <div className="border-b px-4 py-3">
            {selectedTrace ? (
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <div className="text-xs uppercase text-muted-foreground">
                    {t('TRACES_WATERFALL.HEADER')}
                  </div>
                  <div className="flex items-center gap-2 text-sm font-semibold">
                    <span className="font-mono">{selectedTrace.trace_id}</span>
                    <ArrowRight className="h-4 w-4 text-muted-foreground" />
                    <span className="truncate">{selectedTrace.name}</span>
                  </div>
                </div>
                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                  <div className="flex items-center gap-1">
                    <Timer className="h-3.5 w-3.5" />
                    <span>{formatDuration(selectedTrace.duration_ms)}</span>
                  </div>
                  <div>{formatTime(selectedTrace.start_time)}</div>
                </div>
              </div>
            ) : (
              <div className="text-sm text-muted-foreground">{t('TRACES_WATERFALL.EMPTY')}</div>
            )}
          </div>
          <ScrollArea className="flex-1">
            <div className="min-w-[640px] px-4 py-3">
              {spanRows.length === 0 ? (
                <div className="py-12 text-sm text-muted-foreground">
                  {t('TRACES_WATERFALL.NO_SPANS')}
                </div>
              ) : (
                <div className="flex flex-col gap-3">
                  {spanRows.map((row) => (
                    <div key={row.span.span_id} className="grid grid-cols-[240px_1fr] gap-3">
                      <div
                        className="truncate text-xs text-muted-foreground"
                        style={{ paddingLeft: row.depth * 14 }}
                      >
                        <div className="text-[11px] uppercase">{row.span.name}</div>
                        <div className="font-mono text-[10px]">
                          {formatDuration(row.span.duration_ms)}
                        </div>
                      </div>
                      <div className="relative flex items-center">
                        <div className="h-2 w-full rounded-full bg-muted/60" />
                        <div
                          className="absolute h-2 rounded-full bg-sky-500/80"
                          style={{
                            left: `${row.offsetPct}%`,
                            width: `${row.widthPct}%`,
                          }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </ScrollArea>
        </div>
      </div>
    </div>
  )
}
