import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useRealmRecoverySettings } from '@/features/realm/api/useRealmRecoverySettings.ts'
import { useUpdateRealmRecoverySettings } from '@/features/realm/api/useUpdateRealmRecoverySettings.ts'
import {
  type RecoverySettingsSchema,
  recoverySettingsSchema,
} from '@/features/realm/schema/recovery-settings.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
} from '@/shared/ui/form.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'
import { Switch } from '@/shared/ui/switch'
import { FormTextarea } from '@/shared/ui/form-textarea'

const templateHint =
  'Supported tokens: {realm}, {identifier}, {token}, {resume_url}, {expires_at}'

export function RecoverySettingsForm() {
  const { data: realm } = useCurrentRealm()
  const { data: settings, isLoading } = useRealmRecoverySettings()
  const updateMutation = useUpdateRealmRecoverySettings(realm?.id || '')

  const form = useForm<RecoverySettingsSchema>({
    resolver: zodResolver(recoverySettingsSchema) as Resolver<RecoverySettingsSchema>,
    defaultValues: {
      token_ttl_minutes: 15,
      rate_limit_max: 5,
      rate_limit_window_minutes: 15,
      revoke_sessions_on_reset: true,
      email_subject: '',
      email_body: '',
    },
  })

  useEffect(() => {
    if (!settings) return
    form.reset({
      token_ttl_minutes: settings.token_ttl_minutes,
      rate_limit_max: settings.rate_limit_max,
      rate_limit_window_minutes: settings.rate_limit_window_minutes,
      revoke_sessions_on_reset: settings.revoke_sessions_on_reset,
      email_subject: settings.email_subject ?? '',
      email_body: settings.email_body ?? '',
    })
  }, [settings, form])

  const onSubmit = (values: RecoverySettingsSchema) => {
    updateMutation.mutate(
      {
        token_ttl_minutes: values.token_ttl_minutes,
        rate_limit_max: values.rate_limit_max,
        rate_limit_window_minutes: values.rate_limit_window_minutes,
        revoke_sessions_on_reset: values.revoke_sessions_on_reset,
        email_subject: values.email_subject || null,
        email_body: values.email_body || null,
      },
      {
        onSuccess: (data) =>
          form.reset({
            token_ttl_minutes: data.token_ttl_minutes,
            rate_limit_max: data.rate_limit_max,
            rate_limit_window_minutes: data.rate_limit_window_minutes,
            revoke_sessions_on_reset: data.revoke_sessions_on_reset,
            email_subject: data.email_subject ?? '',
            email_body: data.email_body ?? '',
          }),
      },
    )
  }

  useFormPersistence(form, onSubmit, updateMutation.isPending)

  if (isLoading) return <div>Loading settings...</div>
  if (!realm) return <div>Realm not found</div>

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card>
          <CardHeader>
            <CardTitle>Recovery Tokens</CardTitle>
            <CardDescription>Control token lifetime and throttling.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
              <div id="recovery-token-ttl" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="token_ttl_minutes"
                  label="Token TTL (Minutes)"
                  description="How long recovery tokens remain valid."
                  type="number"
                />
              </div>
              <div id="recovery-rate-limit-max" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="rate_limit_max"
                  label="Rate Limit Max"
                  description="Requests per window. Use 0 to disable."
                  type="number"
                />
              </div>
              <div id="recovery-rate-limit-window" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="rate_limit_window_minutes"
                  label="Rate Limit Window (Minutes)"
                  description="Window for rate limiting."
                  type="number"
                />
              </div>
            </div>
            <div id="recovery-revoke-sessions" className="scroll-mt-24 rounded-md -m-2 p-2">
              <FormField
                control={form.control}
                name="revoke_sessions_on_reset"
                render={({ field }) => (
                  <FormItem className="flex items-center justify-between gap-6">
                    <div className="space-y-1">
                      <FormLabel>Revoke Sessions on Reset</FormLabel>
                      <FormDescription>
                        End all active sessions after a successful password reset.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={field.onChange}
                        aria-label="Revoke sessions on reset"
                        disabled={updateMutation.isPending}
                      />
                    </FormControl>
                  </FormItem>
                )}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Recovery Email Template</CardTitle>
            <CardDescription>
              Customize the subject/body sent via SMTP. {templateHint}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div id="recovery-email-subject" className="scroll-mt-24 rounded-md -m-2 p-2">
              <FormInput
                control={form.control}
                name="email_subject"
                label="Email Subject"
                description={templateHint}
              />
            </div>
            <div id="recovery-email-body" className="scroll-mt-24 rounded-md -m-2 p-2">
              <FormTextarea
                control={form.control}
                name="email_body"
                label="Email Body"
                placeholder="Include {resume_url} so users can continue."
                description={templateHint}
                rows={8}
              />
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  )
}
