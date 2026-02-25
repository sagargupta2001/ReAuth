import { useCallback, useEffect, useMemo, useState } from 'react'

import type { OnChangeFn, PaginationState, SortingState } from '@tanstack/react-table'
import { Pause, Play, RotateCw, Sparkles } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import { Command, CommandInput } from '@/components/command'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import type { LogEntry } from '@/entities/log/model/types'
import { useLogStream } from '@/features/logs/hooks/useLogStream'
import { cn } from '@/lib/utils'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'
import { enumParam, numberParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'

import { useTelemetryLogs } from '../api/useTelemetryLogs'
import { useTelemetryLogTargets } from '../api/useTelemetryLogTargets'
import { createLogColumns } from './LogsTableColumns'
import { useIncludeSpansPreference } from '../lib/observabilityPreferences'
import { isWithinRange } from '../lib/timeRange'
import type { ResolvedTimeRange } from '../lib/timeRange'
import type { TelemetryLog } from '../model/types'

interface LogsExplorerProps {
  timeRange: ResolvedTimeRange
  onSelectTrace: (traceId: string) => void
  onRefreshTimeRange?: () => void
}

const LOG_LEVELS = ['all', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] as const
const SORT_FIELDS = ['timestamp', 'duration_ms'] as const
const SORT_DIRS = ['desc', 'asc'] as const

type LogLevelFilter = (typeof LOG_LEVELS)[number]

type SortField = (typeof SORT_FIELDS)[number]

type SortDir = (typeof SORT_DIRS)[number]

function isTraceSpan(log: TelemetryLog) {
  return log.target === 'trace.span' || log.message === 'trace.span'
}

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
    log.fields && typeof log.fields === 'object' && !Array.isArray(log.fields)
      ? log.fields
      : {}

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

export function LogsExplorer({
  timeRange,
  onSelectTrace,
  onRefreshTimeRange,
}: LogsExplorerProps) {
  const { t } = useTranslation('logs')

  const [urlState, setUrlState] = useUrlState<{
    log_page: number
    log_per_page: number
    log_level: LogLevelFilter
    log_source: string
    log_q: string
    log_sort_by: SortField
    log_sort_dir: SortDir
  }>({
    log_page: numberParam(1),
    log_per_page: numberParam(100),
    log_level: enumParam(LOG_LEVELS, 'all'),
    log_source: stringParam('all'),
    log_q: stringParam(''),
    log_sort_by: enumParam(SORT_FIELDS, 'timestamp'),
    log_sort_dir: enumParam(SORT_DIRS, 'desc'),
  })

  const [searchInput, setSearchInput] = useState(urlState.log_q)
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null)

  const levelFilter = urlState.log_level
  const moduleFilter = urlState.log_source
  const { includeSpans } = useIncludeSpansPreference()
  const columns = useMemo(
    () => createLogColumns({ t, onSelectTrace }),
    [onSelectTrace, t],
  )

  useEffect(() => {
    setSearchInput(urlState.log_q)
  }, [urlState.log_q])

  useEffect(() => {
    const handle = window.setTimeout(() => {
      const trimmed = searchInput.trim()
      if (trimmed !== urlState.log_q) {
        setUrlState({ log_q: trimmed, log_page: 1 })
      }
    }, 350)
    return () => window.clearTimeout(handle)
  }, [searchInput, setUrlState, urlState.log_q])

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
      search: urlState.log_q || undefined,
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
    search: urlState.log_q || undefined,
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

  const searchTerm = urlState.log_q.toLowerCase()
  const applyRangeToLive = !(liveAllowed && isConnected)
  const filteredLiveLogs = useMemo(() => {
    return normalizedLive.filter((log) => {
      if (!includeSpans && isTraceSpan(log)) return false
      if (levelFilter !== 'all' && log.level !== levelFilter) return false
      if (moduleFilter !== 'all' && log.target !== moduleFilter) return false
      if (applyRangeToLive && !isWithinRange(log.timestamp, timeRange)) return false
      if (!searchTerm) return true
      const haystack = [
        log.message,
        log.target,
        log.trace_id,
        log.request_id,
        JSON.stringify(log.fields ?? {}),
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      return haystack.includes(searchTerm)
    })
  }, [
    applyRangeToLive,
    includeSpans,
    levelFilter,
    moduleFilter,
    normalizedLive,
    searchTerm,
    timeRange,
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

  const moduleOptions = useMemo(() => {
    const targets = new Set<string>()
    const sourceList =
      targetOptions && targetOptions.length > 0
        ? targetOptions
        : combinedLogs.map((log) => log.target).filter(Boolean)

    sourceList.forEach((target) => {
      if (target) targets.add(target)
    })
    if (moduleFilter && moduleFilter !== 'all') {
      targets.add(moduleFilter)
    }
    return ['all', ...Array.from(targets).slice(0, 12)]
  }, [combinedLogs, moduleFilter, targetOptions])
  const showModuleFilter = moduleOptions.length > 2

  const totalResults = meta?.total ?? combinedLogs.length
  const totalPages = meta?.total_pages && meta.total_pages > 0 ? meta.total_pages : 1
  const pagination = useMemo<PaginationState>(
    () => ({
      pageIndex: Math.max(0, urlState.log_page - 1),
      pageSize: urlState.log_per_page,
    }),
    [urlState.log_page, urlState.log_per_page],
  )

  const sorting = useMemo<SortingState>(
    () => [
      {
        id: urlState.log_sort_by,
        desc: urlState.log_sort_dir === 'desc',
      },
    ],
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
    <div className="flex h-full min-h-0 flex-col gap-4">
      <div className="flex flex-wrap items-center gap-3">
        <Command className="min-w-[240px] flex-1 border bg-background/60">
          <CommandInput
            value={searchInput}
            onValueChange={setSearchInput}
            placeholder={t('LOGS_EXPLORER.SEARCH_PLACEHOLDER')}
            className="h-10 text-sm"
          />
        </Command>
        <Select
          value={levelFilter}
          onValueChange={(value) =>
            setUrlState({ log_level: value as LogLevelFilter, log_page: 1 })
          }
        >
          <SelectTrigger className="w-[160px]">
            <SelectValue placeholder={t('LOGS_EXPLORER.LEVEL_FILTER')} />
          </SelectTrigger>
          <SelectContent>
            {LOG_LEVELS.map((level) => (
              <SelectItem key={level} value={level}>
                {level === 'all' ? t('LOGS_EXPLORER.ALL_LEVELS') : level}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {showModuleFilter && (
          <Select
            value={moduleFilter}
            onValueChange={(value) => setUrlState({ log_source: value, log_page: 1 })}
          >
            <SelectTrigger className="w-[240px]">
              <SelectValue placeholder={t('LOGS_EXPLORER.SOURCE_FILTER')} />
            </SelectTrigger>
            <SelectContent>
              {moduleOptions.map((module) => (
                <SelectItem key={module} value={module}>
                  {module === 'all' ? t('LOGS_EXPLORER.ALL_SOURCES') : module}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        <div className="flex flex-wrap items-center gap-2">
          <div className="flex flex-col gap-1">
            <Button
              variant={isConnected ? 'secondary' : 'outline'}
              onClick={isConnected ? disconnect : connect}
              className="h-11 gap-2"
              disabled={!liveAllowed}
            >
              <span
                className={cn(
                  'h-2 w-2 rounded-full bg-muted-foreground/40',
                  isConnected &&
                    liveAllowed &&
                    'animate-pulse bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.8)]',
                )}
              />
              {isConnected ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
              {t('LOGS_EXPLORER.LIVE_TRAIL')}
            </Button>
            {!liveAllowed && (
              <span className="text-[10px] text-muted-foreground">
                {t('LOGS_EXPLORER.LIVE_TRAIL_HINT')}
              </span>
            )}
          </div>
          <Button
            variant="outline"
            className="h-11 gap-2"
            onClick={handleRefresh}
            disabled={isLoading || isFetching}
          >
            <RotateCw className={cn('h-4 w-4', isFetching && 'animate-spin')} />
          </Button>
        </div>
        <div className="ml-auto flex items-center gap-2 text-xs text-muted-foreground">
          <Sparkles className="h-3.5 w-3.5" />
          {t('LOGS_EXPLORER.RESULT_COUNT', { count: totalResults })}
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
          className="min-h-0 flex-1"
          pageSizeOptions={[50, 100, 200]}
          onRowClick={(log) =>
            setExpandedLogId((current) => (current === log.id ? null : log.id))
          }
          getRowClassName={(log) => (expandedLogId === log.id ? 'bg-muted/40' : '')}
          isRowExpanded={(log) => expandedLogId === log.id}
          renderSubRow={(log) => (
            <div className="bg-muted/30 p-4">
              <div className="flex flex-col gap-2">
                <div className="text-xs font-medium text-muted-foreground">
                  {t('LOGS_TABLE.METADATA')}
                </div>
                <pre className="max-h-72 overflow-auto rounded-md bg-background/80 p-3 text-xs text-muted-foreground">
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
