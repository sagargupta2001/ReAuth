import { useEffect, useMemo } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
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
import type { WebhookEndpointDetails, WebhookEventGroup } from '@/entities/events/model/types'
import { useUpdateWebhook } from '@/features/events/api/useUpdateWebhook'
import { useUpdateWebhookSubscriptions } from '@/features/events/api/useUpdateWebhookSubscriptions'
import { useWebhookEventCatalog } from '@/features/events/api/useWebhookEventCatalog'
import { WebhookEventSubscriptionPicker } from '@/features/events/components/WebhookEventSubscriptionPicker'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'

const webhookConfigureSchema = z.object({
  url: z.string().trim().min(1, 'Destination URL is required').url('Enter a valid URL'),
  http_method: z.enum(['POST', 'PUT']),
  description: z.string(),
  subscriptions: z.array(z.string()).min(1, 'Select at least one event'),
})

type WebhookConfigureFormValues = z.infer<typeof webhookConfigureSchema>

const EMPTY_EVENT_GROUPS: WebhookEventGroup[] = []
const EMPTY_DEFAULT_EVENTS: string[] = []

interface WebhookConfigureTabProps {
  details: WebhookEndpointDetails
  onSaved?: () => void
}

export function WebhookConfigureTab({ details, onSaved }: WebhookConfigureTabProps) {
  const updateWebhook = useUpdateWebhook()
  const updateSubscriptions = useUpdateWebhookSubscriptions()
  const eventCatalog = useWebhookEventCatalog()
  const eventGroups = eventCatalog.data?.groups ?? EMPTY_EVENT_GROUPS
  const defaultEvents = eventCatalog.data?.default_events ?? EMPTY_DEFAULT_EVENTS
  const allEvents = useMemo(
    () => eventGroups.flatMap((group) => group.events.map((event) => event.event_type)),
    [eventGroups],
  )

  const form = useForm<WebhookConfigureFormValues>({
    resolver: zodResolver(webhookConfigureSchema),
    defaultValues: buildDefaults(details, defaultEvents),
  })

  const selectedEvents = form.watch('subscriptions')
  const sendEverything = allEvents.length > 0 && selectedEvents.length === allEvents.length

  useEffect(() => {
    form.reset(buildDefaults(details, defaultEvents))
  }, [defaultEvents, details, form])

  const setSubscriptions = (subscriptions: string[]) => {
    form.setValue('subscriptions', subscriptions, {
      shouldDirty: true,
      shouldTouch: true,
      shouldValidate: true,
    })
  }

  const handleSendEverything = (checked: boolean) => {
    setSubscriptions(checked ? allEvents : defaultEvents)
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
                      <FormLabel className="sr-only">Subscribed events</FormLabel>
                      <FormDescription className="sr-only">
                        Select granular event groups or forward every event.
                      </FormDescription>
                      <WebhookEventSubscriptionPicker
                        groups={eventGroups}
                        selectedEvents={selectedEvents}
                        onSelectedEventsChange={setSubscriptions}
                        sendEverything={sendEverything}
                        onSendEverythingChange={handleSendEverything}
                        disabled={
                          eventCatalog.isLoading ||
                          updateWebhook.isPending ||
                          updateSubscriptions.isPending
                        }
                      />
                      {eventCatalog.isError ? (
                        <p className="text-destructive text-xs">
                          Failed to load webhook event catalog.
                        </p>
                      ) : null}
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

function buildDefaults(
  details: WebhookEndpointDetails,
  defaultEvents: string[],
): WebhookConfigureFormValues {
  const enabledSubscriptions = details.subscriptions
    .filter((subscription) => subscription.enabled)
    .map((subscription) => subscription.event_type)

  return {
    url: details.endpoint.url,
    http_method: details.endpoint.http_method?.toUpperCase() === 'PUT' ? 'PUT' : 'POST',
    description: details.endpoint.description ?? '',
    subscriptions: enabledSubscriptions.length > 0 ? enabledSubscriptions : defaultEvents,
  }
}
