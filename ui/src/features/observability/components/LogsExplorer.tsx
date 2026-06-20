import { useCallback, useEffect, useMemo, useState } from 'react'

import type { OnChangeFn, PaginationState, SortingState } from '@tanstack/react-table'
import { Pause, Play, RotateCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import type { LogEntry } from '@/entities/log/model/types'
import { useLogStream } from '@/features/logs/hooks/useLogStream'
import { cn } from '@/lib/utils'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'
import { enumParam, numberParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'

import { useTelemetryLogs } from '../api/useTelemetryLogs'
import { useTelemetryLogTargets } from '../api/useTelemetryLogTargets'
import { LogSelectFilterChip, LogTextFilterChip } from './LogFilterChips'
import { LogTimeRangeControl } from './LogTimeRangeControl'
import { createLogColumns } from './LogsTableColumns'
import { useIncludeSpansPreference } from '../lib/observabilityPreferences'
import { TIME_RANGE_OPTIONS, isWithinRange } from '../lib/timeRange'
import type { ResolvedTimeRange, TimeRangeKey } from '../lib/timeRange'
import type { TelemetryLog } from '../model/types'

interface LogsExplorerProps {
  timeRange: ResolvedTimeRange
  onSelectTrace: (traceId: string) => void
  onRefreshTimeRange?: () => void
}

const LOG_LEVELS = ['all', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] as const
const SORT_FIELDS = ['timestamp', 'duration_ms'] as const
const SORT_DIRS = ['desc', 'asc'] as const
const TIME_RANGE_KEYS = TIME_RANGE_OPTIONS.map((option) => option.key) as TimeRangeKey[]

const LEVEL_OPTIONS = LOG_LEVELS.filter((level) => level !== 'all').map((level) => ({
  value: level,
  label: level,
}))

type LogLevelFilter = (typeof LOG_LEVELS)[number]
type SortField = (typeof SORT_FIELDS)[number]
type SortDir = (typeof SORT_DIRS)[number]

const isTraceSpan = (log: TelemetryLog) =>
  log.target === 'trace.span' || log.message === 'trace.span'

function parseNumber(value?: string) {
  if (!value) return undefined
  const parsed = Number(value)
  return Number.isNaN(parsed) ? undefined : parsed
}

function normalizeLiveLog(log: LogEntry, index: number): TelemetryLog {
  const fields = log.fields ?? {}
  return {
    id: `live-${log.timestamp}-${index}`,
    timestamp: log.timestamp,
    level: log.level,
    target: log.target,
    message: log.message,
    fields,
    request_id: fields.request_id,
    trace_id: fields.trace_id,
    span_id: fields.span_id,
    parent_id: fields.parent_id,
    user_id: fields.user_id,
    realm: fields.realm,
    method: fields.method,
    route: fields.route,
    path: fields.path,
    status: parseNumber(fields.status),
    duration_ms: parseNumber(fields.duration_ms),
    source: 'live',
  }
}

function normalizeStoredLog(log: TelemetryLog): TelemetryLog {
  const fields =
    log.fields && typeof log.fields === 'object' && !Array.isArray(log.fields) ? log.fields : {}

  return {
    ...log,
    fields,
    source: log.source ?? 'stored',
  }
}

function buildMetadata(log: TelemetryLog) {
  const metadata: Record<string, unknown> = { ...log.fields }
  const add = (key: string, value: unknown) => {
    if (value === null || value === undefined || value === '') return
    metadata[key] = value
  }
  add('request_id', log.request_id)
  add('trace_id', log.trace_id)
  add('span_id', log.span_id)
  add('parent_id', log.parent_id)
  add('user_id', log.user_id)
  add('realm', log.realm)
  add('method', log.method)
  add('route', log.route)
  add('path', log.path)
  add('status', log.status)
  add('duration_ms', log.duration_ms)
  return metadata
}

export function LogsExplorer({ timeRange, onSelectTrace, onRefreshTimeRange }: LogsExplorerProps) {
  const { t } = useTranslation('logs')

  const [urlState, setUrlState] = useUrlState<{
    log_page: number
    log_per_page: number
    log_level: LogLevelFilter
    log_source: string
    log_user: string
    log_trace: string
    log_sort_by: SortField
    log_sort_dir: SortDir
    range: TimeRangeKey
    start: string
    end: string
  }>({
    log_page: numberParam(1),
    log_per_page: numberParam(100),
    log_level: enumParam(LOG_LEVELS, 'all'),
    log_source: stringParam('all'),
    log_user: stringParam(''),
    log_trace: stringParam(''),
    log_sort_by: enumParam(SORT_FIELDS, 'timestamp'),
    log_sort_dir: enumParam(SORT_DIRS, 'desc'),
    range: enumParam(TIME_RANGE_KEYS, '15m'),
    start: stringParam(''),
    end: stringParam(''),
  })

  const [expandedLogId, setExpandedLogId] = useState<string | null>(null)

  const levelFilter = urlState.log_level
  const moduleFilter = urlState.log_source
  const { includeSpans } = useIncludeSpansPreference()
  const columns = useMemo(() => createLogColumns({ t, onSelectTrace }), [onSelectTrace, t])

  const liveAllowed =
    urlState.log_page === 1 &&
    urlState.log_sort_by === 'timestamp' &&
    urlState.log_sort_dir === 'desc'

  const start = timeRange.start ? timeRange.start.toISOString() : undefined
  const end = timeRange.end ? timeRange.end.toISOString() : undefined

  const { logs: liveLogs, isConnected, connect, disconnect } = useLogStream()
  const shouldPollStored = !(isConnected && liveAllowed)
  const { data, isLoading, isFetching, refetch } = useTelemetryLogs(
    {
      level: levelFilter === 'all' ? undefined : levelFilter,
      target: moduleFilter === 'all' ? undefined : moduleFilter,
      user_id: urlState.log_user || undefined,
      trace_id: urlState.log_trace || undefined,
      start,
      end,
      include_spans: includeSpans,
      page: urlState.log_page,
      per_page: urlState.log_per_page,
      sort_by: urlState.log_sort_by,
      sort_dir: urlState.log_sort_dir,
    },
    { enabled: shouldPollStored },
  )

  const { data: targetOptions } = useTelemetryLogTargets({
    level: levelFilter === 'all' ? undefined : levelFilter,
    start,
    end,
    include_spans: includeSpans,
  })

  const meta = data?.meta

  const normalizedStored = useMemo(() => {
    const storedLogs = data?.data ?? []
    return storedLogs.map(normalizeStoredLog)
  }, [data?.data])

  useEffect(() => {
    if (!liveAllowed && isConnected) {
      disconnect()
    }
  }, [disconnect, isConnected, liveAllowed])

  const normalizedLive = useMemo(
    () => (liveAllowed ? liveLogs.map((log, index) => normalizeLiveLog(log, index)) : []),
    [liveAllowed, liveLogs],
  )

  const applyRangeToLive = !(liveAllowed && isConnected)
  const filteredLiveLogs = useMemo(() => {
    return normalizedLive.filter((log) => {
      if (!includeSpans && isTraceSpan(log)) return false
      if (levelFilter !== 'all' && log.level !== levelFilter) return false
      if (moduleFilter !== 'all' && log.target !== moduleFilter) return false
      if (urlState.log_user && log.user_id !== urlState.log_user) return false
      if (urlState.log_trace && log.trace_id !== urlState.log_trace) return false
      if (applyRangeToLive && !isWithinRange(log.timestamp, timeRange)) return false
      return true
    })
  }, [
    applyRangeToLive,
    includeSpans,
    levelFilter,
    moduleFilter,
    normalizedLive,
    timeRange,
    urlState.log_trace,
    urlState.log_user,
  ])

  const handleRefresh = useCallback(() => {
    if (timeRange.key === 'custom') {
      void refetch()
      return
    }
    onRefreshTimeRange?.()
  }, [onRefreshTimeRange, refetch, timeRange.key])

  const combinedLogs = useMemo(() => {
    if (!liveAllowed) {
      return normalizedStored
    }

    const map = new Map<string, TelemetryLog>()
    const addLog = (log: TelemetryLog) => {
      const key = `${log.timestamp}|${log.level}|${log.target}|${log.message}|${log.trace_id ?? ''}`
      if (!map.has(key)) {
        map.set(key, log)
      }
    }
    filteredLiveLogs.forEach(addLog)
    normalizedStored.forEach(addLog)
    const merged = Array.from(map.values())
    merged.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
    return merged
  }, [filteredLiveLogs, liveAllowed, normalizedStored])

  const sourceOptions = useMemo(() => {
    const targets = new Set<string>()
    const sourceList =
      targetOptions && targetOptions.length > 0
        ? targetOptions
        : combinedLogs.map((log) => log.target).filter(Boolean)
    sourceList.forEach((target) => {
      if (target) targets.add(target)
    })
    if (moduleFilter && moduleFilter !== 'all') targets.add(moduleFilter)
    return Array.from(targets)
      .slice(0, 50)
      .map((target) => ({ value: target, label: target }))
  }, [combinedLogs, moduleFilter, targetOptions])

  const handleTimeRangeChange = (next: { rangeKey: TimeRangeKey; start: string; end: string }) => {
    setUrlState({ range: next.rangeKey, start: next.start, end: next.end, log_page: 1 })
  }

  const totalPages = meta?.total_pages && meta.total_pages > 0 ? meta.total_pages : 1
  const pagination = useMemo<PaginationState>(
    () => ({
      pageIndex: Math.max(0, urlState.log_page - 1),
      pageSize: urlState.log_per_page,
    }),
    [urlState.log_page, urlState.log_per_page],
  )

  const sorting = useMemo<SortingState>(
    () => [{ id: urlState.log_sort_by, desc: urlState.log_sort_dir === 'desc' }],
    [urlState.log_sort_by, urlState.log_sort_dir],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setUrlState({
      log_page: nextState.pageIndex + 1,
      log_per_page: nextState.pageSize,
    })
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(sorting) : updater
    if (!nextState.length) {
      setUrlState({ log_sort_by: 'timestamp', log_sort_dir: 'desc', log_page: 1 })
      return
    }
    const primary = nextState[0]
    setUrlState({
      log_sort_by: primary.id as SortField,
      log_sort_dir: primary.desc ? 'desc' : 'asc',
      log_page: 1,
    })
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="flex flex-wrap items-center gap-2">
        <LogSelectFilterChip
          label={t('LOGS_EXPLORER.LEVEL_FILTER')}
          value={levelFilter === 'all' ? '' : levelFilter}
          options={LEVEL_OPTIONS}
          onChange={(value) =>
            setUrlState({ log_level: (value as LogLevelFilter) || 'all', log_page: 1 })
          }
        />
        <LogSelectFilterChip
          label={t('LOGS_EXPLORER.SOURCE_FILTER')}
          value={moduleFilter === 'all' ? '' : moduleFilter}
          options={sourceOptions}
          searchable
          contentClassName="w-96"
          onChange={(value) => setUrlState({ log_source: value || 'all', log_page: 1 })}
        />
        <LogTextFilterChip
          label={t('LOGS_EXPLORER.USER_FILTER')}
          value={urlState.log_user}
          placeholder="User id…"
          onApply={(value) => setUrlState({ log_user: value, log_page: 1 })}
        />
        <LogTextFilterChip
          label={t('LOGS_EXPLORER.TRACE_FILTER')}
          value={urlState.log_trace}
          placeholder="Trace id…"
          onApply={(value) => setUrlState({ log_trace: value, log_page: 1 })}
        />

        <div className="ml-auto flex items-center gap-2">
          <LogTimeRangeControl
            rangeKey={urlState.range}
            start={urlState.start}
            end={urlState.end}
            onChange={handleTimeRangeChange}
          />
          <Button
            variant={isConnected ? 'secondary' : 'outline'}
            onClick={isConnected ? disconnect : connect}
            className="h-9 gap-2"
            disabled={!liveAllowed}
          >
            {isConnected ? (
              <Pause className="h-4 w-4 text-emerald-500" />
            ) : (
              <Play className="text-muted-foreground h-4 w-4" />
            )}
            {t('LOGS_EXPLORER.LIVE_TRAIL')}
          </Button>
          <Button
            variant="outline"
            className="h-9 gap-2"
            onClick={handleRefresh}
            disabled={isLoading || isFetching}
          >
            <RotateCw className={cn('h-4 w-4', isFetching && 'animate-spin')} />
          </Button>
        </div>
      </div>

      {isLoading && combinedLogs.length === 0 ? (
        <DataTableSkeleton columnCount={8} rowCount={10} />
      ) : (
        <DataTable
          columns={columns}
          data={combinedLogs}
          pageCount={totalPages}
          pagination={pagination}
          onPaginationChange={handlePaginationChange}
          sorting={sorting}
          onSortingChange={handleSortingChange}
          showToolbar={false}
          rootClassName="min-h-0 flex-1"
          className="max-h-[calc(100vh-240px)]"
          tableClassName="table-auto min-w-[900px]"
          pageSizeOptions={[50, 100, 200]}
          onRowClick={(log) => setExpandedLogId((current) => (current === log.id ? null : log.id))}
          getRowClassName={(log) => (expandedLogId === log.id ? 'bg-muted/40' : '')}
          isRowExpanded={(log) => expandedLogId === log.id}
          renderSubRow={(log) => (
            <div className="bg-muted/30 p-4">
              <div className="flex flex-col gap-2">
                <div className="text-muted-foreground text-xs font-medium">
                  {t('LOGS_TABLE.METADATA')}
                </div>
                <pre className="bg-background/80 text-muted-foreground max-h-72 overflow-auto rounded-md p-3 text-xs">
                  {JSON.stringify(buildMetadata(log), null, 2)}
                </pre>
              </div>
            </div>
          )}
        />
      )}
    </div>
  )
}
