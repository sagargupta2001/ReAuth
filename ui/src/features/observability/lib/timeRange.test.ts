import { describe, it, expect } from 'vitest'

import {
  combineDateAndTime,
  formatRangeLabel,
  resolveTimeRange,
  timeStringFromDate,
} from './timeRange'

describe('timeRange helpers', () => {
  it('combineDateAndTime sets the local time onto the day', () => {
    const day = new Date(2026, 5, 20, 3, 15) // Jun 20 2026, 03:15 local
    const combined = combineDateAndTime(day, '19:41')
    expect(combined.getFullYear()).toBe(2026)
    expect(combined.getMonth()).toBe(5)
    expect(combined.getDate()).toBe(20)
    expect(combined.getHours()).toBe(19)
    expect(combined.getMinutes()).toBe(41)
    expect(combined.getSeconds()).toBe(0)
  })

  it('combineDateAndTime keeps the day time on malformed input', () => {
    const day = new Date(2026, 5, 20, 8, 30)
    const combined = combineDateAndTime(day, 'not-a-time')
    expect(combined.getHours()).toBe(8)
    expect(combined.getMinutes()).toBe(30)
  })

  it('timeStringFromDate pads to HH:MM', () => {
    expect(timeStringFromDate(new Date(2026, 5, 20, 7, 5))).toBe('07:05')
    expect(timeStringFromDate(new Date(2026, 5, 20, 19, 41))).toBe('19:41')
  })

  it('formatRangeLabel returns the preset label for non-custom ranges', () => {
    const now = new Date(2026, 5, 20, 19, 41)
    const range = resolveTimeRange('24h', undefined, now)
    expect(formatRangeLabel(range)).toBe('Last 24h')
  })

  it('custom range passes start/end through inclusive of time', () => {
    const start = new Date(2026, 5, 20, 0, 0).toISOString()
    const end = new Date(2026, 5, 20, 19, 41).toISOString()
    const range = resolveTimeRange('custom', { start, end })
    expect(range.start?.toISOString()).toBe(start)
    expect(range.end?.toISOString()).toBe(end)
    expect(formatRangeLabel(range)).toContain('–')
  })
})
