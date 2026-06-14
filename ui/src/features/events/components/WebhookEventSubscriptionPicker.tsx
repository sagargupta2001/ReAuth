import { useMemo, useState } from 'react'

import type { CheckedState } from '@radix-ui/react-checkbox'
import { ChevronDown, ChevronRight, Search } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Checkbox } from '@/components/checkbox'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/collapsible'
import { Input } from '@/components/input'
import { ScrollArea } from '@/components/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import type { WebhookEventGroup } from '@/entities/events/model/types'
import { cn } from '@/lib/utils'

type CategoryFilter = 'all' | string

interface WebhookEventSubscriptionPickerProps {
  groups: WebhookEventGroup[]
  selectedEvents: string[]
  onSelectedEventsChange: (events: string[]) => void
  sendEverything: boolean
  onSendEverythingChange: (checked: boolean) => void
  disabled?: boolean
  className?: string
}

export function WebhookEventSubscriptionPicker({
  groups,
  selectedEvents,
  onSelectedEventsChange,
  sendEverything,
  onSendEverythingChange,
  disabled = false,
  className,
}: WebhookEventSubscriptionPickerProps) {
  const [query, setQuery] = useState('')
  const [category, setCategory] = useState<CategoryFilter>('all')
  const [openGroups, setOpenGroups] = useState<Set<string>>(
    () => new Set(groups.map((group) => group.id)),
  )

  const allEventTypes = useMemo(
    () => groups.flatMap((group) => group.events.map((event) => event.event_type)),
    [groups],
  )
  const selectedSet = useMemo(() => new Set(selectedEvents), [selectedEvents])

  const filteredGroups = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()

    return groups
      .filter((group) => category === 'all' || group.id === category)
      .map((group) => ({
        ...group,
        events: group.events.filter((event) => {
          if (!normalizedQuery) return true
          return [event.event_type, event.label, event.description].some((value) =>
            value.toLowerCase().includes(normalizedQuery),
          )
        }),
      }))
      .filter((group) => group.events.length > 0)
  }, [category, groups, query])

  const setSelected = (events: Iterable<string>) => {
    onSelectedEventsChange(Array.from(new Set(events)))
  }

  const toggleEvent = (eventType: string, checked: CheckedState) => {
    const next = new Set(selectedSet)
    if (checked === true) {
      next.add(eventType)
    } else {
      next.delete(eventType)
    }
    setSelected(next)
  }

  const toggleGroup = (eventTypes: string[], checked: CheckedState) => {
    const next = new Set(selectedSet)
    if (checked === true) {
      eventTypes.forEach((eventType) => next.add(eventType))
    } else {
      eventTypes.forEach((eventType) => next.delete(eventType))
    }
    setSelected(next)
  }

  const groupState = (eventTypes: string[]): CheckedState => {
    const selectedCount = eventTypes.filter((eventType) => selectedSet.has(eventType)).length
    if (selectedCount === 0) return false
    if (selectedCount === eventTypes.length) return true
    return 'indeterminate'
  }

  const setGroupOpen = (groupId: string, open: boolean) => {
    setOpenGroups((prev) => {
      const next = new Set(prev)
      if (open) {
        next.add(groupId)
      } else {
        next.delete(groupId)
      }
      return next
    })
  }

  const handleSendEverything = (checked: boolean) => {
    onSendEverythingChange(checked)
    if (checked) setSelected(allEventTypes)
  }

  return (
    <div className={cn('overflow-hidden rounded-lg border', className)}>
      <div className="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3">
        <div>
          <p className="text-sm font-semibold">Event Subscriptions</p>
          <p className="text-muted-foreground text-xs">
            {selectedSet.size} of {allEventTypes.length} events selected
          </p>
        </div>
        <label className="flex items-center gap-2 text-xs">
          <span className="text-muted-foreground">Send all events</span>
          <Switch
            checked={sendEverything}
            onCheckedChange={handleSendEverything}
            disabled={disabled}
          />
        </label>
      </div>

      <div className="bg-muted/20 grid gap-3 border-b p-3 lg:grid-cols-[1fr_210px_auto]">
        <div className="relative">
          <Search className="text-muted-foreground/60 absolute top-2.5 left-3 h-4 w-4" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search events..."
            className="h-9 border pl-9 text-sm"
          />
        </div>

        <Select value={category} onValueChange={setCategory}>
          <SelectTrigger className="h-9">
            <SelectValue placeholder="Filter by category" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All categories</SelectItem>
            {groups.map((group) => (
              <SelectItem key={group.id} value={group.id}>
                {group.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => {
            onSendEverythingChange(false)
            onSelectedEventsChange([])
          }}
          disabled={disabled || selectedSet.size === 0}
        >
          Clear all
        </Button>
      </div>

      <ScrollArea className="h-[360px]">
        <div className="space-y-2 p-3">
          {filteredGroups.length === 0 ? (
            <div className="text-muted-foreground rounded-md border border-dashed px-3 py-8 text-center text-xs">
              No events match this view.
            </div>
          ) : (
            filteredGroups.map((group) => {
              const eventTypes = group.events.map((event) => event.event_type)
              const selectedCount = eventTypes.filter((eventType) =>
                selectedSet.has(eventType),
              ).length
              const isOpen = openGroups.has(group.id)

              return (
                <Collapsible
                  key={group.id}
                  open={isOpen}
                  onOpenChange={(open) => setGroupOpen(group.id, open)}
                  className="bg-background/60 overflow-hidden rounded-md border"
                >
                  <div className="flex items-center gap-3 border-b px-3 py-2">
                    <CollapsibleTrigger asChild>
                      <Button type="button" variant="ghost" size="icon" className="h-6 w-6">
                        {isOpen ? (
                          <ChevronDown className="h-4 w-4" />
                        ) : (
                          <ChevronRight className="h-4 w-4" />
                        )}
                      </Button>
                    </CollapsibleTrigger>
                    <Checkbox
                      checked={groupState(eventTypes)}
                      onCheckedChange={(checked) => toggleGroup(eventTypes, checked)}
                      disabled={disabled || sendEverything}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-sm font-semibold">{group.label}</span>
                        <Badge variant="neutralMuted" className="text-[10px]">
                          {selectedCount} of {group.events.length} selected
                        </Badge>
                      </div>
                      <p className="text-muted-foreground truncate text-xs">{group.description}</p>
                    </div>
                  </div>

                  <CollapsibleContent>
                    <div className="grid gap-2 p-3 md:grid-cols-2 xl:grid-cols-3">
                      {group.events.map((event) => (
                        <label
                          key={event.event_type}
                          className={cn(
                            'flex min-w-0 cursor-pointer items-start gap-2 rounded-md border p-3 transition',
                            selectedSet.has(event.event_type)
                              ? 'border-primary/30 bg-primary/10'
                              : 'border-border/60 hover:border-primary/20 hover:bg-muted/40',
                            (disabled || sendEverything) && 'cursor-not-allowed opacity-60',
                          )}
                        >
                          <Checkbox
                            checked={selectedSet.has(event.event_type)}
                            onCheckedChange={(checked) => toggleEvent(event.event_type, checked)}
                            disabled={disabled || sendEverything}
                            className="mt-0.5"
                          />
                          <span className="min-w-0">
                            <span className="block truncate font-mono text-xs font-medium">
                              {event.event_type}
                            </span>
                            <span className="text-muted-foreground mt-1 line-clamp-2 block text-xs">
                              {event.description}
                            </span>
                          </span>
                        </label>
                      ))}
                    </div>
                  </CollapsibleContent>
                </Collapsible>
              )
            })
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
