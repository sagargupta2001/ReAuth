import { useMemo, useState } from 'react'

import { CalendarClock, ChevronDown } from 'lucide-react'

import { Button } from '@/components/button'
import { Calendar } from '@/components/calendar'
import { Input } from '@/components/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/popover'
import { cn } from '@/lib/utils'

import {
  TIME_RANGE_OPTIONS,
  combineDateAndTime,
  formatRangeLabel,
  resolveTimeRange,
  timeStringFromDate,
} from '../lib/timeRange'
import type { TimeRangeKey } from '../lib/timeRange'

interface LogTimeRangeControlProps {
  rangeKey: TimeRangeKey
  start: string
  end: string
  onChange: (next: { rangeKey: TimeRangeKey; start: string; end: string }) => void
}

const PRESETS = TIME_RANGE_OPTIONS.filter((option) => option.key !== 'custom')

interface DraftRange {
  from?: Date
  to?: Date
}

export function LogTimeRangeControl({ rangeKey, start, end, onChange }: LogTimeRangeControlProps) {
  const [open, setOpen] = useState(false)

  const resolved = useMemo(
    () => resolveTimeRange(rangeKey, { start: start || undefined, end: end || undefined }),
    [rangeKey, start, end],
  )

  // Draft state (committed on Apply). Seeded from the active range whenever the popover opens.
  const [draftKey, setDraftKey] = useState<TimeRangeKey>(rangeKey)
  const [draftRange, setDraftRange] = useState<DraftRange>({})
  const [fromTime, setFromTime] = useState('00:00')
  const [toTime, setToTime] = useState('23:59')

  const seedFromActive = () => {
    const current = resolveTimeRange(rangeKey, { start: start || undefined, end: end || undefined })
    const now = new Date()
    const from = current.start ?? new Date(now.getTime() - 60 * 60 * 1000)
    const to = current.end ?? now
    setDraftKey(rangeKey)
    setDraftRange({ from, to })
    setFromTime(timeStringFromDate(from))
    setToTime(timeStringFromDate(to))
  }

  const handleOpenChange = (next: boolean) => {
    if (next) seedFromActive()
    setOpen(next)
  }

  const selectPreset = (key: TimeRangeKey) => {
    const preview = resolveTimeRange(key)
    setDraftKey(key)
    if (preview.start && preview.end) {
      setDraftRange({ from: preview.start, to: preview.end })
      setFromTime(timeStringFromDate(preview.start))
      setToTime(timeStringFromDate(preview.end))
    }
  }

  const apply = () => {
    if (draftKey !== 'custom') {
      onChange({ rangeKey: draftKey, start: '', end: '' })
      setOpen(false)
      return
    }
    const fromDay = draftRange.from
    const toDay = draftRange.to ?? draftRange.from
    if (!fromDay || !toDay) {
      setOpen(false)
      return
    }
    const from = combineDateAndTime(fromDay, fromTime)
    const to = combineDateAndTime(toDay, toTime)
    onChange({ rangeKey: 'custom', start: from.toISOString(), end: to.toISOString() })
    setOpen(false)
  }

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button variant="outline" className="h-9 gap-2">
          <CalendarClock className="h-4 w-4" />
          {formatRangeLabel(resolved)}
          <ChevronDown className="h-3.5 w-3.5 opacity-60" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-auto p-0">
        <div className="flex">
          {/* Presets */}
          <div className="flex w-40 flex-col gap-0.5 border-r p-2">
            {PRESETS.map((preset) => (
              <button
                key={preset.key}
                type="button"
                onClick={() => selectPreset(preset.key)}
                className={cn(
                  'hover:bg-muted rounded-md px-3 py-2 text-left text-sm transition-colors',
                  draftKey === preset.key && 'bg-muted text-foreground font-medium',
                )}
              >
                {preset.label.replace('Last ', 'Last ')}
              </button>
            ))}
            <button
              type="button"
              onClick={() => setDraftKey('custom')}
              className={cn(
                'hover:bg-muted rounded-md px-3 py-2 text-left text-sm transition-colors',
                draftKey === 'custom' && 'bg-muted text-foreground font-medium',
              )}
            >
              Custom
            </button>
          </div>

          {/* Custom day + time */}
          <div className="flex flex-col gap-3 p-3">
            <div className="flex items-center gap-2 text-xs">
              <div className="flex flex-col gap-1">
                <span className="text-muted-foreground">From</span>
                <Input
                  type="time"
                  value={fromTime}
                  onChange={(event) => {
                    setFromTime(event.target.value)
                    setDraftKey('custom')
                  }}
                  className="h-8 w-28"
                />
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-muted-foreground">To</span>
                <Input
                  type="time"
                  value={toTime}
                  onChange={(event) => {
                    setToTime(event.target.value)
                    setDraftKey('custom')
                  }}
                  className="h-8 w-28"
                />
              </div>
            </div>

            <Calendar
              mode="range"
              numberOfMonths={1}
              selected={draftRange as { from: Date; to?: Date }}
              defaultMonth={draftRange.from}
              onSelect={(value: { from?: Date; to?: Date } | undefined) => {
                setDraftKey('custom')
                setDraftRange({ from: value?.from, to: value?.to })
              }}
            />

            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
                Cancel
              </Button>
              <Button size="sm" onClick={apply}>
                Apply
              </Button>
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
