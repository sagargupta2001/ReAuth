import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useRealmSecurityHeaders } from '@/features/realm/api/useRealmSecurityHeaders.ts'
import { useUpdateRealmSecurityHeaders } from '@/features/realm/api/useUpdateRealmSecurityHeaders.ts'
import {
  type SecurityHeadersSchema,
  securityHeadersSchema,
} from '@/features/realm/schema/security-headers.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'

const headerHint = 'Leave blank to disable a header for this realm.'

export function SecurityHeadersSettingsForm() {
  const { data: realm } = useCurrentRealm()
  const { data: settings, isLoading } = useRealmSecurityHeaders()
  const updateMutation = useUpdateRealmSecurityHeaders(realm?.id || '')

  const form = useForm<SecurityHeadersSchema>({
    resolver: zodResolver(securityHeadersSchema) as Resolver<SecurityHeadersSchema>,
    defaultValues: {
      x_frame_options: 'SAMEORIGIN',
      content_security_policy: "frame-ancestors 'self'",
      x_content_type_options: 'nosniff',
      referrer_policy: 'no-referrer',
      strict_transport_security: '',
    },
  })

  useEffect(() => {
    if (!settings) return
    form.reset({
      x_frame_options: settings.x_frame_options ?? '',
      content_security_policy: settings.content_security_policy ?? '',
      x_content_type_options: settings.x_content_type_options ?? '',
      referrer_policy: settings.referrer_policy ?? '',
      strict_transport_security: settings.strict_transport_security ?? '',
    })
  }, [settings, form])

  const onSubmit = (values: SecurityHeadersSchema) => {
    updateMutation.mutate(
      {
        x_frame_options: values.x_frame_options || null,
        content_security_policy: values.content_security_policy || null,
        x_content_type_options: values.x_content_type_options || null,
        referrer_policy: values.referrer_policy || null,
        strict_transport_security: values.strict_transport_security || null,
      },
      {
        onSuccess: (data) =>
          form.reset({
            x_frame_options: data.x_frame_options ?? '',
            content_security_policy: data.content_security_policy ?? '',
            x_content_type_options: data.x_content_type_options ?? '',
            referrer_policy: data.referrer_policy ?? '',
            strict_transport_security: data.strict_transport_security ?? '',
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
            <CardTitle>Browser Security Headers</CardTitle>
            <CardDescription>
              Apply security headers to auth and OIDC responses. {headerHint}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div id="security-x-frame-options" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="x_frame_options"
                  label="X-Frame-Options"
                  description="Controls embedding in frames. Example: SAMEORIGIN"
                />
              </div>
              <div
                id="security-content-security-policy"
                className="scroll-mt-24 rounded-md -m-2 p-2"
              >
                <FormInput
                  control={form.control}
                  name="content_security_policy"
                  label="Content-Security-Policy"
                  description="Restrict where the UI can be embedded."
                  placeholder="frame-ancestors 'self'"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Additional Headers</CardTitle>
            <CardDescription>Harden common response headers. {headerHint}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div
                id="security-x-content-type-options"
                className="scroll-mt-24 rounded-md -m-2 p-2"
              >
                <FormInput
                  control={form.control}
                  name="x_content_type_options"
                  label="X-Content-Type-Options"
                  description="Disable MIME sniffing. Example: nosniff"
                />
              </div>
              <div
                id="security-referrer-policy"
                className="scroll-mt-24 rounded-md -m-2 p-2"
              >
                <FormInput
                  control={form.control}
                  name="referrer_policy"
                  label="Referrer-Policy"
                  description="Control referrer data. Example: no-referrer"
                />
              </div>
              <div
                id="security-strict-transport-security"
                className="scroll-mt-24 rounded-md -m-2 p-2"
              >
                <FormInput
                  control={form.control}
                  name="strict_transport_security"
                  label="Strict-Transport-Security"
                  description="Enable HSTS only on HTTPS. Example: max-age=31536000"
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  )
}
