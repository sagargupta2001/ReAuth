import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useRealmIdpSettings } from '@/features/realm/api/useRealmIdpSettings.ts'
import { useUpdateRealmIdpSettings } from '@/features/realm/api/useUpdateRealmIdpSettings.ts'
import { RealmSettingsCard } from '@/features/realm/components/RealmSettingsCard'
import {
  type IdpSettingsSchema,
  idpSettingsSchema,
} from '@/features/realm/schema/idp-settings.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'

export function IdpSettingsForm() {
  const { data: realm } = useCurrentRealm()
  const { data: settings, isLoading } = useRealmIdpSettings()
  const updateMutation = useUpdateRealmIdpSettings(realm?.id || '')

  const form = useForm<IdpSettingsSchema>({
    resolver: zodResolver(idpSettingsSchema) as Resolver<IdpSettingsSchema>,
    defaultValues: {
      oauth_start_rate_limit_max: 30,
      oauth_start_rate_limit_window_minutes: 10,
    },
  })

  useEffect(() => {
    if (!settings) return
    form.reset({
      oauth_start_rate_limit_max: settings.oauth_start_rate_limit_max,
      oauth_start_rate_limit_window_minutes: settings.oauth_start_rate_limit_window_minutes,
    })
  }, [settings, form])

  const onSubmit = (values: IdpSettingsSchema) => {
    updateMutation.mutate(values, {
      onSuccess: (data) =>
        form.reset({
          oauth_start_rate_limit_max: data.oauth_start_rate_limit_max,
          oauth_start_rate_limit_window_minutes: data.oauth_start_rate_limit_window_minutes,
        }),
    })
  }

  useFormPersistence(form, onSubmit, updateMutation.isPending)

  if (isLoading) return <div>Loading settings...</div>
  if (!realm) return <div>Realm not found</div>

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <RealmSettingsCard
          title="OAuth Broker Start Throttling"
          description="Limit the OAuth broker start endpoint per client IP and provider inside this realm."
          bodyClassName="space-y-6"
        >
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <div id="idp-start-rate-limit-max" className="-m-2 scroll-mt-24 rounded-md p-2">
              <FormInput
                control={form.control}
                name="oauth_start_rate_limit_max"
                label="Start Rate Limit Max"
                description="Starts per IP per provider. Use 0 to disable the dedicated IdP throttle."
                type="number"
              />
            </div>
            <div id="idp-start-rate-limit-window" className="-m-2 scroll-mt-24 rounded-md p-2">
              <FormInput
                control={form.control}
                name="oauth_start_rate_limit_window_minutes"
                label="Start Rate Limit Window (Minutes)"
                description="Window length for the per-provider OAuth start throttle."
                type="number"
              />
            </div>
          </div>
        </RealmSettingsCard>
      </form>
    </Form>
  )
}
