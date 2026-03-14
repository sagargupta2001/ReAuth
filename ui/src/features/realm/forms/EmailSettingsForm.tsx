import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useRealmEmailSettings } from '@/features/realm/api/useRealmEmailSettings.ts'
import { useUpdateRealmEmailSettings } from '@/features/realm/api/useUpdateRealmEmailSettings.ts'
import {
  type EmailSettingsSchema,
  emailSettingsSchema,
} from '@/features/realm/schema/email-settings.schema.ts'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

export function EmailSettingsForm() {
  const { data: realm } = useCurrentRealm()
  const { data: settings, isLoading } = useRealmEmailSettings()
  const updateMutation = useUpdateRealmEmailSettings(realm?.id || '')

  const form = useForm<EmailSettingsSchema>({
    resolver: zodResolver(emailSettingsSchema) as Resolver<EmailSettingsSchema>,
    defaultValues: {
      enabled: false,
      from_address: '',
      from_name: '',
      reply_to_address: '',
      smtp_host: '',
      smtp_port: 587,
      smtp_username: '',
      smtp_password: '',
      smtp_security: 'starttls',
    },
  })

  useEffect(() => {
    if (!settings) return
    form.reset({
      enabled: settings.enabled,
      from_address: settings.from_address ?? '',
      from_name: settings.from_name ?? '',
      reply_to_address: settings.reply_to_address ?? '',
      smtp_host: settings.smtp_host ?? '',
      smtp_port: settings.smtp_port ?? 587,
      smtp_username: settings.smtp_username ?? '',
      smtp_password: '',
      smtp_security: settings.smtp_security ?? 'starttls',
    })
  }, [settings, form])

  const onSubmit = (values: EmailSettingsSchema) => {
    const payload = {
      enabled: values.enabled,
      from_address: values.from_address || null,
      from_name: values.from_name || null,
      reply_to_address: values.reply_to_address || null,
      smtp_host: values.smtp_host || null,
      smtp_port: values.smtp_port || null,
      smtp_username: values.smtp_username || null,
      smtp_security: values.smtp_security,
      ...(values.smtp_password
        ? {
            smtp_password: values.smtp_password,
          }
        : {}),
    }

    updateMutation.mutate(payload, {
      onSuccess: (data) =>
        form.reset({
          enabled: data.enabled,
          from_address: data.from_address ?? '',
          from_name: data.from_name ?? '',
          reply_to_address: data.reply_to_address ?? '',
          smtp_host: data.smtp_host ?? '',
          smtp_port: data.smtp_port ?? 587,
          smtp_username: data.smtp_username ?? '',
          smtp_password: '',
          smtp_security: data.smtp_security ?? 'starttls',
        }),
    })
  }

  useFormPersistence(form, onSubmit, updateMutation.isPending)

  if (isLoading) return <div>Loading settings...</div>
  if (!realm) return <div>Realm not found</div>

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card>
          <CardHeader>
            <CardTitle>Email Delivery</CardTitle>
            <CardDescription>Configure SMTP delivery for recovery and verification.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div id="email-enabled" className="scroll-mt-24 rounded-md -m-2 p-2">
              <FormField
                control={form.control}
                name="enabled"
                render={({ field }) => (
                  <FormItem className="flex items-center justify-between gap-6">
                    <div className="space-y-1">
                      <FormLabel>Enable Email Delivery</FormLabel>
                      <FormDescription>Turn on SMTP-based emails for this realm.</FormDescription>
                    </div>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={field.onChange}
                        aria-label="Enable email delivery"
                        disabled={updateMutation.isPending}
                      />
                    </FormControl>
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div id="email-from-address" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="from_address"
                  label="From Address"
                  description="Address shown in the From field."
                />
              </div>
              <div id="email-from-name" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="from_name"
                  label="From Name"
                  description="Optional display name."
                />
              </div>
              <div id="email-reply-to" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="reply_to_address"
                  label="Reply-To Address"
                  description="Optional reply-to override."
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>SMTP Server</CardTitle>
            <CardDescription>Set the SMTP connection details.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div id="email-smtp-host" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="smtp_host"
                  label="SMTP Host"
                  description="Hostname or IP address."
                />
              </div>
              <div id="email-smtp-port" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="smtp_port"
                  label="SMTP Port"
                  description="Usually 587 for STARTTLS, 465 for TLS."
                  type="number"
                />
              </div>
              <div id="email-smtp-security" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormField
                  control={form.control}
                  name="smtp_security"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Security</FormLabel>
                      <FormControl>
                        <Select value={field.value} onValueChange={field.onChange}>
                          <SelectTrigger>
                            <SelectValue placeholder="Select TLS mode" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="starttls">STARTTLS</SelectItem>
                            <SelectItem value="tls">TLS (SMTPS)</SelectItem>
                            <SelectItem value="none">None</SelectItem>
                          </SelectContent>
                        </Select>
                      </FormControl>
                      <FormDescription>Use STARTTLS when supported.</FormDescription>
                    </FormItem>
                  )}
                />
              </div>
              <div id="email-smtp-username" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="smtp_username"
                  label="SMTP Username"
                  description="Leave blank if your server does not require auth."
                />
              </div>
              <div id="email-smtp-password" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="smtp_password"
                  label="SMTP Password"
                  description={
                    settings?.smtp_password_set
                      ? 'Password is already set. Enter a new one to rotate.'
                      : 'Set the password for SMTP authentication.'
                  }
                  type="password"
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  )
}
