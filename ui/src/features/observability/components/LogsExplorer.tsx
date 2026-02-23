import { Fragment, useEffect, useMemo, useState } from 'react'

import { ChevronLeft, ChevronRight, Pause, Play, Sparkles } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Command, CommandInput } from '@/components/command'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/table'
import type { LogEntry } from '@/entities/log/model/types'
import { useLogStream } from '@/features/logs/hooks/useLogStream'
import { cn } from '@/lib/utils'
import { enumParam, numberParam, stringParam, useUrlState } from '@/shared/lib/hooks/useUrlState'

import { useTelemetryLogs } from '../api/useTelemetryLogs'
import { isWithinRange } from '../lib/timeRange'
import type { ResolvedTimeRange } from '../lib/timeRange'
import type { TelemetryLog } from '../model/types'

interface LogsExplorerProps {
  timeRange: ResolvedTimeRange
  onSelectTrace: (traceId: string) => void
}

const LOG_LEVELS = ['all', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] as const
const SORT_FIELDS = ['timestamp', 'duration_ms', 'status', 'level'] as const
const SORT_DIRS = ['desc', 'asc'] as const
const PER_PAGE_OPTIONS = [50, 100, 200]

type LogLevelFilter = (typeof LOG_LEVELS)[number]

type SortField = (typeof SORT_FIELDS)[number]

type SortDir = (typeof SORT_DIRS)[number]

type LogSortOption = {
  value: string
  label: string
  sort_by: SortField
  sort_dir: SortDir
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

function getDisplayMessage(log: TelemetryLog): string {
  if (log.message) return log.message
  if (typeof log.fields.message === 'string') return log.fields.message
  if (typeof log.fields.msg === 'string') return log.fields.msg
  return 'No message, view metadata for details.'
}

function formatTimestamp(timestamp: string) {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  return date.toLocaleString('en-US', {
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

function levelBadgeVariant(level: string) {
  switch (level) {
    case 'ERROR':
      return 'destructive'
    case 'WARN':
      return 'secondary'
    case 'INFO':
      return 'default'
    default:
      return 'secondary'
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

export function LogsExplorer({ timeRange, onSelectTrace }: LogsExplorerProps) {
  const { t } = useTranslation('logs')
  const sortOptions: LogSortOption[] = useMemo(
    () => [
      {
        value: 'timestamp:desc',
        label: t('LOGS_EXPLORER.SORT_NEWEST'),
        sort_by: 'timestamp',
        sort_dir: 'desc',
      },
      {
        value: 'timestamp:asc',
        label: t('LOGS_EXPLORER.SORT_OLDEST'),
        sort_by: 'timestamp',
        sort_dir: 'asc',
      },
      {
        value: 'duration_ms:desc',
        label: t('LOGS_EXPLORER.SORT_SLOWEST'),
        sort_by: 'duration_ms',
        sort_dir: 'desc',
      },
      {
        value: 'duration_ms:asc',
        label: t('LOGS_EXPLORER.SORT_FASTEST'),
        sort_by: 'duration_ms',
        sort_dir: 'asc',
      },
    ],
    [t],
  )

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
  const sortValue = `${urlState.log_sort_by}:${urlState.log_sort_dir}`

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

  const start = timeRange.start ? timeRange.start.toISOString() : undefined
  const end = timeRange.end ? timeRange.end.toISOString() : undefined

  const { data, isLoading } = useTelemetryLogs({
    level: levelFilter === 'all' ? undefined : levelFilter,
    target: moduleFilter === 'all' ? undefined : moduleFilter,
    search: urlState.log_q || undefined,
    start,
    end,
    page: urlState.log_page,
    per_page: urlState.log_per_page,
    sort_by: urlState.log_sort_by,
    sort_dir: urlState.log_sort_dir,
  })

  const storedLogs = data?.data ?? []
  const meta = data?.meta

  const normalizedStored = useMemo(() => storedLogs.map(normalizeStoredLog), [storedLogs])

  const { logs: liveLogs, isConnected, connect, disconnect } = useLogStream()
  const liveAllowed =
    urlState.log_page === 1 &&
    urlState.log_sort_by === 'timestamp' &&
    urlState.log_sort_dir === 'desc'

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
  const filteredLiveLogs = useMemo(() => {
    return normalizedLive.filter((log) => {
      if (levelFilter !== 'all' && log.level !== levelFilter) return false
      if (moduleFilter !== 'all' && log.target !== moduleFilter) return false
      if (!isWithinRange(log.timestamp, timeRange)) return false
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
  }, [levelFilter, moduleFilter, normalizedLive, searchTerm, timeRange])

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
    combinedLogs.forEach((log) => {
      if (log.target) targets.add(log.target)
    })
    if (moduleFilter && moduleFilter !== 'all') {
      targets.add(moduleFilter)
    }
    return ['all', ...Array.from(targets).slice(0, 12)]
  }, [combinedLogs, moduleFilter])

  const totalResults = meta?.total ?? combinedLogs.length
  const totalPages = meta?.total_pages && meta.total_pages > 0 ? meta.total_pages : 1
  const isFirstPage = urlState.log_page <= 1
  const isLastPage = totalPages > 0 ? urlState.log_page >= totalPages : true

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-col gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <Command className="flex-1 border bg-background/60">
            <CommandInput
              value={searchInput}
              onValueChange={setSearchInput}
              placeholder={t('LOGS_EXPLORER.SEARCH_PLACEHOLDER')}
              className="h-11 text-sm"
            />
          </Command>
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
        </div>
        <div className="flex flex-wrap items-center gap-3">
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
          <Select
            value={sortValue}
            onValueChange={(value) => {
              const option = sortOptions.find((item) => item.value === value)
              if (!option) return
              setUrlState({
                log_sort_by: option.sort_by,
                log_sort_dir: option.sort_dir,
                log_page: 1,
              })
            }}
          >
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder={t('LOGS_EXPLORER.SORT_LABEL')} />
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
            <Sparkles className="h-3.5 w-3.5" />
            {t('LOGS_EXPLORER.RESULT_COUNT', { count: totalResults })}
          </div>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-hidden rounded-lg border bg-background/40">
        <div className="h-full overflow-auto">
          <Table>
            <TableHeader className="bg-muted/30 sticky top-0 z-10">
              <TableRow>
                <TableHead className="w-[180px]">{t('LOGS_TABLE.TIMESTAMP')}</TableHead>
                <TableHead className="w-[110px]">{t('LOGS_TABLE.LEVEL')}</TableHead>
                <TableHead className="w-[220px]">{t('LOGS_TABLE.TRACE_ID')}</TableHead>
                <TableHead>{t('LOGS_TABLE.MESSAGE')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading && combinedLogs.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="py-12 text-center text-muted-foreground">
                    {t('LOGS_TABLE.LOADING')}
                  </TableCell>
                </TableRow>
              ) : combinedLogs.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={4} className="py-12 text-center text-muted-foreground">
                    {t('LOGS_TABLE.EMPTY')}
                  </TableCell>
                </TableRow>
              ) : (
                combinedLogs.map((log) => {
                  const isExpanded = expandedLogId === log.id
                  return (
                    <Fragment key={log.id}>
                      <TableRow
                        className={cn('cursor-pointer transition-colors', {
                          'bg-muted/40': isExpanded,
                        })}
                        onClick={() =>
                          setExpandedLogId((current) => (current === log.id ? null : log.id))
                        }
                      >
                        <TableCell className="font-mono text-xs text-muted-foreground">
                          {formatTimestamp(log.timestamp)}
                        </TableCell>
                        <TableCell>
                          <Badge variant={levelBadgeVariant(log.level)} className="text-xs">
                            {log.level}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          {log.trace_id ? (
                            <button
                              className="font-mono text-xs text-sky-400 hover:text-sky-300"
                              onClick={(event) => {
                                event.stopPropagation()
                                onSelectTrace(log.trace_id ?? '')
                              }}
                            >
                              {log.trace_id}
                            </button>
                          ) : (
                            <span className="text-xs text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell className="text-sm">
                          <div className="flex flex-col gap-1">
                            <span className="font-medium text-foreground">
                              {getDisplayMessage(log)}
                            </span>
                            <span className="text-xs text-muted-foreground truncate">
                              {log.target}
                            </span>
                          </div>
                        </TableCell>
                      </TableRow>
                      {isExpanded && (
                        <TableRow className="bg-muted/30">
                          <TableCell colSpan={4} className="p-4">
                            <div className="flex flex-col gap-2">
                              <div className="text-xs font-medium text-muted-foreground">
                                {t('LOGS_TABLE.METADATA')}
                              </div>
                              <pre className="max-h-72 overflow-auto rounded-md bg-background/80 p-3 text-xs text-muted-foreground">
                                {JSON.stringify(buildMetadata(log), null, 2)}
                              </pre>
                            </div>
                          </TableCell>
                        </TableRow>
                      )}
                    </Fragment>
                  )
                })
              )}
            </TableBody>
          </Table>
        </div>
        <div className="flex flex-wrap items-center justify-between gap-3 border-t px-4 py-3 text-xs text-muted-foreground">
          <div>
            {t('LOGS_TABLE.PAGE_STATUS', {
              page: urlState.log_page,
              total: totalPages,
            })}
          </div>
          <div className="flex items-center gap-2">
            <span>{t('LOGS_TABLE.ROWS_PER_PAGE')}</span>
            <Select
              value={String(urlState.log_per_page)}
              onValueChange={(value) => setUrlState({ log_per_page: Number(value), log_page: 1 })}
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
              onClick={() => setUrlState({ log_page: Math.max(1, urlState.log_page - 1) })}
              disabled={isFirstPage}
            >
              <ChevronLeft className="h-4 w-4" />
              {t('LOGS_TABLE.PREV')}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setUrlState({ log_page: urlState.log_page + 1 })}
              disabled={isLastPage}
            >
              {t('LOGS_TABLE.NEXT')}
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
