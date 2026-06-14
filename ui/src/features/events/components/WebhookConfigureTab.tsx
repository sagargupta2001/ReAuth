import { useEffect, useMemo } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import type { CheckedState } from '@radix-ui/react-checkbox'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Badge } from '@/components/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Checkbox } from '@/components/checkbox'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/form'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Textarea } from '@/components/textarea'
import type { WebhookEndpointDetails } from '@/entities/events/model/types'
import { useUpdateWebhook } from '@/features/events/api/useUpdateWebhook'
import { useUpdateWebhookSubscriptions } from '@/features/events/api/useUpdateWebhookSubscriptions'
import {
  DEFAULT_WEBHOOK_EVENTS,
  WEBHOOK_EVENT_GROUPS,
} from '@/features/events/model/webhook-events'
import { cn } from '@/lib/utils'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'

const webhookConfigureSchema = z.object({
  url: z.string().trim().min(1, 'Destination URL is required').url('Enter a valid URL'),
  http_method: z.enum(['POST', 'PUT']),
  description: z.string(),
  subscriptions: z.array(z.string()).min(1, 'Select at least one event'),
})

type WebhookConfigureFormValues = z.infer<typeof webhookConfigureSchema>

interface WebhookConfigureTabProps {
  details: WebhookEndpointDetails
  onSaved?: () => void
}

export function WebhookConfigureTab({ details, onSaved }: WebhookConfigureTabProps) {
  const updateWebhook = useUpdateWebhook()
  const updateSubscriptions = useUpdateWebhookSubscriptions()
  const allEvents = useMemo(() => WEBHOOK_EVENT_GROUPS.flatMap((group) => group.events), [])

  const form = useForm<WebhookConfigureFormValues>({
    resolver: zodResolver(webhookConfigureSchema),
    defaultValues: buildDefaults(details),
  })

  const selectedEvents = form.watch('subscriptions')
  const sendEverything = selectedEvents.length === allEvents.length

  useEffect(() => {
    form.reset(buildDefaults(details))
  }, [details, form])

  const setSubscriptions = (subscriptions: string[]) => {
    form.setValue('subscriptions', subscriptions, {
      shouldDirty: true,
      shouldTouch: true,
      shouldValidate: true,
    })
  }

  const toggleEvent = (eventName: string, checked: CheckedState) => {
    const next = new Set(form.getValues('subscriptions'))
    if (checked === true) {
      next.add(eventName)
    } else {
      next.delete(eventName)
    }
    setSubscriptions(Array.from(next))
  }

  const toggleGroup = (events: readonly string[], checked: CheckedState) => {
    const next = new Set(form.getValues('subscriptions'))
    if (checked === true) {
      events.forEach((event) => next.add(event))
    } else {
      events.forEach((event) => next.delete(event))
    }
    setSubscriptions(Array.from(next))
  }

  const groupState = (events: readonly string[]): CheckedState => {
    const selectedCount = events.filter((event) => selectedEvents.includes(event)).length
    if (selectedCount === 0) return false
    if (selectedCount === events.length) return true
    return 'indeterminate'
  }

  const handleSendEverything = (checked: boolean) => {
    setSubscriptions(checked ? allEvents : DEFAULT_WEBHOOK_EVENTS)
  }

  const onSubmit = (values: WebhookConfigureFormValues) => {
    void saveWebhookConfiguration(values)
  }

  const saveWebhookConfiguration = async (values: WebhookConfigureFormValues) => {
    const trimmedUrl = values.url.trim()
    const description = values.description.trim()

    await updateWebhook.mutateAsync({
      endpointId: details.endpoint.id,
      payload: {
        url: trimmedUrl,
        description: description || undefined,
        http_method: values.http_method,
      },
    })

    await updateSubscriptions.mutateAsync({
      endpointId: details.endpoint.id,
      subscriptions: allEvents.map((event) => ({
        event_type: event,
        enabled: values.subscriptions.includes(event),
      })),
    })

    form.reset({
      url: trimmedUrl,
      http_method: values.http_method,
      description,
      subscriptions: values.subscriptions,
    })
    onSaved?.()
  }

  useFormPersistence(form, onSubmit, updateWebhook.isPending || updateSubscriptions.isPending)

  return (
    <div className="max-w-4xl space-y-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Webhook Configuration</CardTitle>
              <CardDescription>
                Edit the destination, method, and description for this target.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground space-y-5 rounded-2xl p-4">
                <div className="grid gap-4 sm:grid-cols-[1fr_140px]">
                  <FormInput
                    control={form.control}
                    name="url"
                    label="Destination URL"
                    placeholder="https://example.com/reauth/webhooks"
                    description="ReAuth will deliver signed webhook payloads to this URL."
                  />

                  <FormField
                    control={form.control}
                    name="http_method"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>HTTP Method</FormLabel>
                        <Select value={field.value} onValueChange={field.onChange}>
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select method" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="POST">POST</SelectItem>
                            <SelectItem value="PUT">PUT</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>

                <FormField
                  control={form.control}
                  name="description"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Description</FormLabel>
                      <FormControl>
                        <Textarea
                          placeholder="Optional notes for this endpoint"
                          className="min-h-[100px] resize-none"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Event Subscriptions</CardTitle>
              <CardDescription>
                Select which event types should be routed to this webhook.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground rounded-2xl p-4">
                <FormField
                  control={form.control}
                  name="subscriptions"
                  render={() => (
                    <FormItem>
                      <div className="bg-background/60 overflow-hidden rounded-lg border border-dashed">
                        <div className="flex items-center justify-between gap-4 border-b p-4">
                          <div>
                            <FormLabel>Subscribed events</FormLabel>
                            <FormDescription>
                              Select granular event groups or forward every event.
                            </FormDescription>
                          </div>
                          <label className="flex cursor-pointer items-center gap-2 text-sm">
                            <Checkbox
                              checked={sendEverything}
                              onCheckedChange={(checked) => handleSendEverything(checked === true)}
                            />
                            Send everything
                          </label>
                        </div>

                        <div className="space-y-4 p-4">
                          {WEBHOOK_EVENT_GROUPS.map((group) => {
                            const state = groupState(group.events)
                            return (
                              <div key={group.id} className="space-y-3">
                                <div className="flex items-start justify-between gap-3">
                                  <div>
                                    <div className="flex items-center gap-2">
                                      <Checkbox
                                        checked={state}
                                        onCheckedChange={(checked) =>
                                          toggleGroup(group.events, checked)
                                        }
                                      />
                                      <span className="text-sm font-semibold">{group.label}</span>
                                    </div>
                                    <p className="text-muted-foreground text-xs">
                                      {group.description}
                                    </p>
                                  </div>
                                  <Badge variant="outline" className="text-xs">
                                    {group.events.length} events
                                  </Badge>
                                </div>

                                <div className="grid gap-2 pl-6 sm:grid-cols-2">
                                  {group.events.map((event) => (
                                    <label
                                      key={event}
                                      className={cn(
                                        'text-muted-foreground flex items-center gap-2 text-sm',
                                        selectedEvents.includes(event) && 'text-foreground',
                                      )}
                                    >
                                      <Checkbox
                                        checked={selectedEvents.includes(event)}
                                        onCheckedChange={(checked) => toggleEvent(event, checked)}
                                      />
                                      <span className="font-mono text-xs">{event}</span>
                                    </label>
                                  ))}
                                </div>
                              </div>
                            )
                          })}
                        </div>
                      </div>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>
    </div>
  )
}

function buildDefaults(details: WebhookEndpointDetails): WebhookConfigureFormValues {
  const enabledSubscriptions = details.subscriptions
    .filter((subscription) => subscription.enabled)
    .map((subscription) => subscription.event_type)

  return {
    url: details.endpoint.url,
    http_method: details.endpoint.http_method?.toUpperCase() === 'PUT' ? 'PUT' : 'POST',
    description: details.endpoint.description ?? '',
    subscriptions: enabledSubscriptions,
  }
}
