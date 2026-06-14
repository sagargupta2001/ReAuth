import { useEffect, useMemo, useState } from 'react'

import { Button } from '@/components/button'
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
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Separator } from '@/components/separator'
import { Switch } from '@/components/switch'
import { Textarea } from '@/components/textarea'
import type { WebhookEventGroup } from '@/entities/events/model/types'
import { useCreateWebhook } from '@/features/events/api/useCreateWebhook'
import { useDeleteWebhook } from '@/features/events/api/useDeleteWebhook'
import { useUpdateWebhook } from '@/features/events/api/useUpdateWebhook'
import { useUpdateWebhookSubscriptions } from '@/features/events/api/useUpdateWebhookSubscriptions'
import { useWebhookEventCatalog } from '@/features/events/api/useWebhookEventCatalog'
import { WebhookEventSubscriptionPicker } from '@/features/events/components/WebhookEventSubscriptionPicker'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'

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

const EMPTY_EVENT_GROUPS: WebhookEventGroup[] = []
const EMPTY_DEFAULT_EVENTS: string[] = []

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
  const eventCatalog = useWebhookEventCatalog()
  const eventGroups = eventCatalog.data?.groups ?? EMPTY_EVENT_GROUPS
  const defaultEvents = eventCatalog.data?.default_events ?? EMPTY_DEFAULT_EVENTS
  const allEvents = useMemo(
    () => eventGroups.flatMap((group) => group.events.map((event) => event.event_type)),
    [eventGroups],
  )
  const [open, setOpen] = useState(defaultOpen)
  const [sendEverything, setSendEverything] = useState(false)
  const [selectedEvents, setSelectedEvents] = useState<Set<string>>(() => new Set())
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
        : new Set(defaultEvents)
    setSelectedEvents(initialSet)
    setSendEverything(allEvents.length > 0 && initialSubscriptions?.length === allEvents.length)
    setUrl(initialUrl ?? '')
    setMethod(initialMethod?.toUpperCase() ?? 'POST')
    setDescription(initialDescription ?? '')
  }, [
    open,
    initialSubscriptions,
    initialUrl,
    initialMethod,
    initialDescription,
    allEvents,
    defaultEvents,
  ])

  const resetForm = () => {
    setSendEverything(false)
    setSelectedEvents(new Set(defaultEvents))
    setUrl('')
    setMethod('POST')
    setDescription('')
  }

  const handleSendEverythingChange = (checked: boolean) => {
    setSendEverything(checked)
    setSelectedEvents(new Set(checked ? allEvents : defaultEvents))
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
      <DialogContent className="sm:max-w-[640px]">
        <DialogHeader className="px-6 pt-6">
          <DialogTitle>
            {mode === 'create' ? 'Create webhook endpoint' : 'Edit webhook endpoint'}
          </DialogTitle>
          <DialogDescription>
            Configure a destination URL and choose which ReAuth events should be delivered.
          </DialogDescription>
        </DialogHeader>

        <Separator className="my-1" />

        <div className="grid gap-5 px-6 pb-6">
          <div className="grid gap-4 sm:grid-cols-[1fr_140px]">
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

          <div className="space-y-3">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="space-y-1">
                <Label>Event Subscriptions</Label>
                <p className="text-muted-foreground text-xs">
                  {selectedEvents.size} of {allEvents.length} events selected
                </p>
              </div>
              <label className="flex items-center gap-2 pt-1 text-xs">
                <span className="text-muted-foreground">Send all events</span>
                <Switch
                  checked={sendEverything}
                  onCheckedChange={handleSendEverythingChange}
                  disabled={
                    eventCatalog.isLoading ||
                    allEvents.length === 0 ||
                    createWebhook.isPending ||
                    updateWebhook.isPending ||
                    updateSubscriptions.isPending
                  }
                />
              </label>
            </div>

            <WebhookEventSubscriptionPicker
              groups={eventGroups}
              selectedEvents={Array.from(selectedEvents)}
              onSelectedEventsChange={(events) => setSelectedEvents(new Set(events))}
              sendEverything={sendEverything}
              onSendEverythingChange={handleSendEverythingChange}
              disabled={
                eventCatalog.isLoading ||
                createWebhook.isPending ||
                updateWebhook.isPending ||
                updateSubscriptions.isPending
              }
            />
          </div>
          {eventCatalog.isError ? (
            <p className="text-destructive text-xs">Failed to load webhook event catalog.</p>
          ) : null}
        </div>

        <DialogFooter className="gap-1 py-3 pr-3">
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button
            onClick={handleSave}
            disabled={
              !url.trim() ||
              selectedEvents.size === 0 ||
              eventCatalog.isLoading ||
              allEvents.length === 0 ||
              createWebhook.isPending ||
              updateWebhook.isPending ||
              updateSubscriptions.isPending
            }
          >
            {mode === 'create' ? 'Create Webhook' : 'Update Endpoint'}
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
