import { useState } from 'react'

import { CalendarIcon, ChevronDownIcon } from '@radix-ui/react-icons'

import { cn } from '@/lib/utils'
import { Button } from '@/shared/ui/button'
import { Calendar } from '@/shared/ui/calendar'
import { Input } from '@/shared/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/shared/ui/popover'

export interface DateTimeRange {
  from?: Date
  to?: Date
}

interface DateTimeRangePickerProps {
  /** Current value; Date objects or ISO/date strings (e.g. when rehydrated from the URL). */
  value?: { from?: Date | string; to?: Date | string }
  onChange: (range: DateTimeRange) => void
  align?: 'start' | 'center' | 'end'
  triggerClassName?: string
  placeholder?: string
}

function toDate(value?: Date | string): Date | undefined {
  if (!value) return undefined
  const date = value instanceof Date ? value : new Date(value)
  return Number.isNaN(date.getTime()) ? undefined : date
}

function timeOf(date?: Date): string {
  if (!date) return '00:00'
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

function withTime(day: Date, time: string): Date {
  const result = new Date(day)
  const match = /^(\d{1,2}):(\d{2})$/.exec(time.trim())
  if (match) {
    result.setHours(Math.min(23, Number(match[1])), Math.min(59, Number(match[2])), 0, 0)
  }
  return result
}

function formatStamp(date?: Date): string {
  if (!date) return '…'
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function DateTimeRangePicker({
  value,
  onChange,
  align = 'start',
  triggerClassName,
  placeholder = 'Pick a range',
}: DateTimeRangePickerProps) {
  const [open, setOpen] = useState(false)
  const [draft, setDraft] = useState<DateTimeRange>({})
  const [fromTime, setFromTime] = useState('00:00')
  const [toTime, setToTime] = useState('23:59')

  const currentFrom = toDate(value?.from)
  const currentTo = toDate(value?.to)

  const seed = () => {
    setDraft({ from: currentFrom, to: currentTo })
    setFromTime(timeOf(currentFrom))
    setToTime(currentTo ? timeOf(currentTo) : '23:59')
  }

  const handleOpenChange = (next: boolean) => {
    if (next) seed()
    setOpen(next)
  }

  const apply = () => {
    const fromDay = draft.from
    const toDay = draft.to ?? draft.from
    onChange({
      from: fromDay ? withTime(fromDay, fromTime) : undefined,
      to: toDay ? withTime(toDay, toTime) : undefined,
    })
    setOpen(false)
  }

  const label =
    currentFrom || currentTo ? `${formatStamp(currentFrom)} – ${formatStamp(currentTo)}` : placeholder

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn('h-8 justify-between gap-2 px-2 text-xs font-normal', triggerClassName)}
        >
          <span className="flex items-center gap-1.5">
            <CalendarIcon className="h-3.5 w-3.5 opacity-60" />
            {label}
          </span>
          <ChevronDownIcon className="h-3.5 w-3.5 opacity-60" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align={align} className="w-auto p-3">
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-3 text-xs">
            <label className="flex flex-col gap-1">
              <span className="text-muted-foreground">From</span>
              <Input
                type="time"
                value={fromTime}
                onChange={(event) => setFromTime(event.target.value)}
                className="h-8 w-28"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-muted-foreground">To</span>
              <Input
                type="time"
                value={toTime}
                onChange={(event) => setToTime(event.target.value)}
                className="h-8 w-28"
              />
            </label>
          </div>

          <Calendar
            mode="range"
            numberOfMonths={1}
            selected={draft as { from: Date; to?: Date }}
            defaultMonth={draft.from}
            onSelect={(range: { from?: Date; to?: Date } | undefined) =>
              setDraft({ from: range?.from, to: range?.to })
            }
          />

          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={apply} disabled={!draft.from}>
              Apply
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
