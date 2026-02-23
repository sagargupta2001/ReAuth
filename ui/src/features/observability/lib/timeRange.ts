export type TimeRangeKey = '15m' | '1h' | '6h' | '24h' | '7d' | 'custom'

export interface TimeRangeOption {
  key: TimeRangeKey
  label: string
  minutes?: number
}

export interface CustomTimeRange {
  start?: string
  end?: string
}

export interface ResolvedTimeRange {
  key: TimeRangeKey
  label: string
  start: Date | null
  end: Date | null
}

export const TIME_RANGE_OPTIONS: TimeRangeOption[] = [
  { key: '15m', label: 'Last 15m', minutes: 15 },
  { key: '1h', label: 'Last 1h', minutes: 60 },
  { key: '6h', label: 'Last 6h', minutes: 360 },
  { key: '24h', label: 'Last 24h', minutes: 1_440 },
  { key: '7d', label: 'Last 7d', minutes: 10_080 },
  { key: 'custom', label: 'Custom' },
]

export function resolveTimeRange(
  key: TimeRangeKey,
  custom?: CustomTimeRange,
  now: Date = new Date(),
): ResolvedTimeRange {
  const option = TIME_RANGE_OPTIONS.find((item) => item.key === key)
  if (!option) {
    return { key, label: 'Last 15m', start: null, end: null }
  }

  if (key === 'custom') {
    const start = custom?.start ? new Date(custom.start) : null
    const end = custom?.end ? new Date(custom.end) : null
    const invalidStart = start && Number.isNaN(start.getTime())
    const invalidEnd = end && Number.isNaN(end.getTime())
    return {
      key,
      label: option.label,
      start: invalidStart ? null : start,
      end: invalidEnd ? null : end,
    }
  }

  if (!option.minutes) {
    return { key, label: option.label, start: null, end: null }
  }

  const end = now
  const start = new Date(now.getTime() - option.minutes * 60 * 1000)
  return { key, label: option.label, start, end }
}

export function isWithinRange(dateString: string, range: ResolvedTimeRange): boolean {
  if (!range.start || !range.end) {
    return true
  }
  const timestamp = new Date(dateString).getTime()
  if (Number.isNaN(timestamp)) {
    return true
  }
  return timestamp >= range.start.getTime() && timestamp <= range.end.getTime()
}
