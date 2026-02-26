import { useEffect, useMemo, useState } from 'react'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Card, CardContent } from '@/components/card'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/dialog'
import { Checkbox } from '@/components/checkbox'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { ScrollArea } from '@/components/scroll-area'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import { Textarea } from '@/components/textarea'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'
import { useCreateWebhook } from '@/features/events/api/useCreateWebhook'
import { useDeleteWebhook } from '@/features/events/api/useDeleteWebhook'
import { useUpdateWebhook } from '@/features/events/api/useUpdateWebhook'
import { useUpdateWebhookSubscriptions } from '@/features/events/api/useUpdateWebhookSubscriptions'
import { cn } from '@/lib/utils'
import type { CheckedState } from '@radix-ui/react-checkbox'

const EVENT_GROUPS = [
  {
    id: 'users',
    label: 'Users',
    description: 'Authentication and lifecycle changes',
    events: [
      'user.created',
      'user.updated',
      'user.disabled',
      'user.deleted',
      'user.assigned',
      'user.removed',
    ],
  },
  {
    id: 'roles',
    label: 'Roles',
    description: 'Role assignments and permission changes',
    events: ['role.created', 'role.updated', 'role.assigned', 'role.removed', 'role.deleted'],
  },
  {
    id: 'groups',
    label: 'Groups',
    description: 'Group membership changes',
    events: ['group.created', 'group.updated', 'group.assigned', 'group.removed', 'group.deleted'],
  },
]

const DEFAULT_SELECTED_EVENTS = ['user.created', 'user.updated']

interface WebhookEndpointFormProps {
  trigger?: React.ReactNode
  mode?: 'create' | 'edit'
  defaultOpen?: boolean
  endpointId?: string
  initialUrl?: string
  initialMethod?: string
  initialDescription?: string | null
  initialSubscriptions?: string[]
  onSaved?: () => void
}

export function WebhookEndpointForm({
  trigger,
  mode = 'create',
  defaultOpen = false,
  endpointId,
  initialUrl,
  initialMethod,
  initialDescription,
  initialSubscriptions,
  onSaved,
}: WebhookEndpointFormProps) {
  const createWebhook = useCreateWebhook()
  const deleteWebhook = useDeleteWebhook()
  const updateWebhook = useUpdateWebhook()
  const updateSubscriptions = useUpdateWebhookSubscriptions()
  const allEvents = useMemo(() => EVENT_GROUPS.flatMap((group) => group.events), [])
  const [open, setOpen] = useState(defaultOpen)
  const [sendEverything, setSendEverything] = useState(false)
  const [selectedEvents, setSelectedEvents] = useState<Set<string>>(
    () => new Set(DEFAULT_SELECTED_EVENTS),
  )
  const [url, setUrl] = useState('')
  const [method, setMethod] = useState('POST')
  const [description, setDescription] = useState('')
  const [deleteOpen, setDeleteOpen] = useState(false)

  useEffect(() => {
    if (sendEverything) {
      setSelectedEvents(new Set(allEvents))
    }
  }, [sendEverything, allEvents])

  useEffect(() => {
    if (!open) return
    const initialSet =
      initialSubscriptions && initialSubscriptions.length > 0
        ? new Set(initialSubscriptions)
        : new Set(DEFAULT_SELECTED_EVENTS)
    setSelectedEvents(initialSet)
    setSendEverything(initialSubscriptions?.length === allEvents.length)
    setUrl(initialUrl ?? '')
    setMethod(initialMethod?.toUpperCase() ?? 'POST')
    setDescription(initialDescription ?? '')
  }, [open, initialSubscriptions, initialUrl, initialMethod, initialDescription, allEvents])

  const toggleEvent = (eventName: string, checked: CheckedState) => {
    setSelectedEvents((prev) => {
      const next = new Set(prev)
      if (checked === true) {
        next.add(eventName)
      } else {
        next.delete(eventName)
      }
      return next
    })
  }

  const toggleGroup = (events: string[], checked: CheckedState) => {
    setSelectedEvents((prev) => {
      const next = new Set(prev)
      if (checked === true) {
        events.forEach((event) => next.add(event))
      } else {
        events.forEach((event) => next.delete(event))
      }
      return next
    })
  }

  const groupState = (events: string[]): CheckedState => {
    const selectedCount = events.filter((event) => selectedEvents.has(event)).length
    if (selectedCount === 0) return false
    if (selectedCount === events.length) return true
    return 'indeterminate'
  }

  const resetForm = () => {
    setSendEverything(false)
    setSelectedEvents(new Set(DEFAULT_SELECTED_EVENTS))
    setUrl('')
    setMethod('POST')
    setDescription('')
  }

  const handleSave = async () => {
    const trimmedUrl = url.trim()
    try {
      if (!trimmedUrl || selectedEvents.size === 0) return

        if (mode === 'create') {
          const name = deriveEndpointName(trimmedUrl)
          await createWebhook.mutateAsync({
            name,
            url: trimmedUrl,
            description: description.trim() || undefined,
            http_method: method,
            subscriptions: Array.from(selectedEvents),
          })
        } else {
          if (!endpointId) return
          await updateWebhook.mutateAsync({
            endpointId,
            payload: {
              url: trimmedUrl,
              description: description.trim() || undefined,
              http_method: method,
            },
          })

        const toggles = allEvents.map((event) => ({
          event_type: event,
          enabled: selectedEvents.has(event),
        }))
        await updateSubscriptions.mutateAsync({ endpointId, subscriptions: toggles })
      }

      setOpen(false)
      resetForm()
      onSaved?.()
    } catch (err) {
      console.error('Failed to save webhook endpoint', err)
    }
  }

  const handleDelete = async () => {
    if (!endpointId) return
    try {
      await deleteWebhook.mutateAsync(endpointId)
      setDeleteOpen(false)
      setOpen(false)
      resetForm()
      onSaved?.()
    } catch (err) {
      console.error('Failed to delete webhook endpoint', err)
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        setOpen(nextOpen)
        if (!nextOpen) resetForm()
      }}
    >
      {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Configure Webhook Endpoint</DialogTitle>
          <DialogDescription>
            Configure a destination URL and choose which ReAuth events should be delivered.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="webhook-url">Destination URL</Label>
            <Input
              id="webhook-url"
              placeholder="https://example.com/reauth/webhooks"
              value={url}
              onChange={(event) => setUrl(event.target.value)}
            />
          </div>

          <div className="grid gap-2">
            <Label>HTTP Method</Label>
            <Select value={method} onValueChange={setMethod}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select method" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="POST">POST</SelectItem>
                <SelectItem value="PUT">PUT</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="webhook-description">Description</Label>
            <Textarea
              id="webhook-description"
              placeholder="Optional notes for this endpoint"
              className="min-h-[90px]"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
            />
          </div>

          <Card className="border-dashed">
            <CardContent className="flex flex-col gap-4 p-4">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-sm font-medium">Send me everything</p>
                  <p className="text-xs text-muted-foreground">
                    Override granular selections and forward every event.
                  </p>
                </div>
                <Switch checked={sendEverything} onCheckedChange={setSendEverything} />
              </div>

              <ScrollArea className="h-[280px] rounded-md border">
                <div className="space-y-4 p-4">
                  {EVENT_GROUPS.map((group) => {
                    const state = groupState(group.events)
                    return (
                      <div key={group.id} className="space-y-3">
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="flex items-center gap-2">
                              <Checkbox
                                checked={state}
                                onCheckedChange={(checked) => toggleGroup(group.events, checked)}
                                disabled={sendEverything}
                              />
                              <span className="text-sm font-semibold">{group.label}</span>
                            </div>
                            <p className="text-xs text-muted-foreground">{group.description}</p>
                          </div>
                          <Badge variant="outline" className="text-xs">
                            {group.events.length} events
                          </Badge>
                        </div>

                        <div className="grid gap-2 pl-6">
                          {group.events.map((event) => (
                            <label
                              key={event}
                              className={cn(
                                'flex items-center gap-2 text-sm text-muted-foreground',
                                sendEverything && 'opacity-60',
                              )}
                            >
                              <Checkbox
                                checked={selectedEvents.has(event)}
                                onCheckedChange={(checked) => toggleEvent(event, checked)}
                                disabled={sendEverything}
                              />
                              <span className="font-mono text-xs text-foreground">{event}</span>
                            </label>
                          ))}
                        </div>
                      </div>
                    )
                  })}
                </div>
              </ScrollArea>
            </CardContent>
          </Card>
        </div>

        <DialogFooter className="gap-2">
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button
            onClick={handleSave}
            disabled={
              !url.trim() ||
              selectedEvents.size === 0 ||
              createWebhook.isPending ||
              updateWebhook.isPending ||
              updateSubscriptions.isPending
            }
          >
            {mode === 'create' ? 'Save Endpoint' : 'Update Endpoint'}
          </Button>
        </DialogFooter>
      </DialogContent>
      <ConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        title="Delete webhook endpoint"
        desc="This will permanently delete the webhook and all subscriptions."
        destructive
        isLoading={deleteWebhook.isPending}
        handleConfirm={handleDelete}
        confirmText={deleteWebhook.isPending ? 'Deleting...' : 'Delete'}
      />
    </Dialog>
  )
}

function deriveEndpointName(url: string) {
  try {
    const parsed = new URL(url)
    if (parsed.hostname) return parsed.hostname
  } catch {
    // ignore invalid URLs
  }
  return url
}
