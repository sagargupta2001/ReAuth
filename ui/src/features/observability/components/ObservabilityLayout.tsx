import { type ReactNode } from 'react'

import { CalendarClock } from 'lucide-react'

import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { cn } from '@/lib/utils'

import type { CustomTimeRange, TimeRangeKey } from '../lib/timeRange'
import { TIME_RANGE_OPTIONS } from '../lib/timeRange'

export interface ObservabilityTab {
  value: string
  label: string
  content: ReactNode
}

interface ObservabilityLayoutProps {
  title: string
  description: string
  tabs: ObservabilityTab[]
  activeTab: string
  onTabChange: (value: string) => void
  timeRange: TimeRangeKey
  onTimeRangeChange: (value: TimeRangeKey) => void
  customRange: CustomTimeRange
  onCustomRangeChange: (value: CustomTimeRange) => void
  timeRangeLabel?: string
  timeRangePlaceholder?: string
  summary?: ReactNode
}

export function ObservabilityLayout({
  title,
  description,
  tabs,
  activeTab,
  onTabChange,
  timeRange,
  onTimeRangeChange,
  customRange,
  onCustomRangeChange,
  timeRangeLabel,
  timeRangePlaceholder,
  summary,
}: ObservabilityLayoutProps) {
  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
          <p className="text-sm text-muted-foreground">{description}</p>
        </div>
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <CalendarClock className="h-4 w-4" />
            {timeRangeLabel ?? 'Time Range'}
          </div>
          <Select value={timeRange} onValueChange={(value) => onTimeRangeChange(value as TimeRangeKey)}>
            <SelectTrigger className="w-[170px]">
              <SelectValue placeholder={timeRangePlaceholder ?? 'Select range'} />
            </SelectTrigger>
            <SelectContent>
              {TIME_RANGE_OPTIONS.map((option) => (
                <SelectItem key={option.key} value={option.key}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {timeRange === 'custom' && (
            <div className="grid gap-2 text-xs text-muted-foreground">
              <Input
                type="datetime-local"
                value={customRange.start ?? ''}
                onChange={(event) =>
                  onCustomRangeChange({ ...customRange, start: event.target.value })
                }
              />
              <Input
                type="datetime-local"
                value={customRange.end ?? ''}
                onChange={(event) =>
                  onCustomRangeChange({ ...customRange, end: event.target.value })
                }
              />
            </div>
          )}
        </div>
      </div>

      {summary}

      <Tabs value={activeTab} onValueChange={onTabChange} className="flex min-h-0 flex-1 flex-col">
        <TabsList className="w-fit rounded-full border bg-muted/40 p-1">
          {tabs.map((tab) => (
            <TabsTrigger
              key={tab.value}
              value={tab.value}
              className={cn(
                'rounded-full px-4 py-1.5 text-xs font-semibold uppercase tracking-wide',
                'data-[state=active]:bg-background data-[state=active]:shadow-sm',
              )}
            >
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {tabs.map((tab) => (
          <TabsContent key={tab.value} value={tab.value} className="min-h-0 flex-1 pt-4">
            {tab.content}
          </TabsContent>
        ))}
      </Tabs>
    </div>
  )
}
