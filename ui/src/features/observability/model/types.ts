export type LogLevel = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | string

export type LogFields = Record<string, unknown>

export interface TelemetryLog {
  id: string
  timestamp: string
  level: LogLevel
  target: string
  message: string
  fields: LogFields
  request_id?: string | null
  trace_id?: string | null
  span_id?: string | null
  parent_id?: string | null
  user_id?: string | null
  realm?: string | null
  method?: string | null
  route?: string | null
  path?: string | null
  status?: number | null
  duration_ms?: number | null
  source?: 'stored' | 'live'
}

export interface TelemetryTrace {
  trace_id: string
  span_id: string
  parent_id?: string | null
  name: string
  start_time: string
  duration_ms: number
  status?: number | null
  method?: string | null
  route?: string | null
  path?: string | null
  request_id?: string | null
  user_id?: string | null
  realm?: string | null
}

export interface CacheStats {
  namespace: string
  hit_rate: number
  entry_count: number
  max_capacity: number
}

export interface MetricsSnapshot {
  since: string
  request_count: number
  status_counts: {
    success: number
    redirect: number
    client_error: number
    server_error: number
  }
  latency_ms: {
    buckets: Array<{ le: number; count: number }>
    overflow_count: number
    count: number
    sum_ms: number
    avg_ms: number
  }
}
