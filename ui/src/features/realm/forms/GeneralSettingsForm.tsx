import { useEffect, useRef } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'
import { toast } from 'sonner'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useUpdateRealm } from '@/features/realm/api/useUpdateRealm.ts'
import { useUpdateRealmOptimistic } from '@/features/realm/api/useUpdateRealmOptimistic'
import {
  type GeneralSettingsSchema,
  generalSettingsSchema,
} from '@/features/realm/schema/setting.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'
import { Switch } from '@/shared/ui/switch'

export function GeneralSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()
  const updateMutation = useUpdateRealm(realm?.id || '')
  const toggleMutation = useUpdateRealmOptimistic(realm?.id || '', realm?.name || '')
  const registrationFlowIdRef = useRef<string | null>(null)

  const form = useForm<GeneralSettingsSchema>({
    resolver: zodResolver(generalSettingsSchema) as Resolver<GeneralSettingsSchema>,
    defaultValues: {
      name: '',
    },
  })

  const onSubmit = (values: GeneralSettingsSchema) => {
    updateMutation.mutate(values, {
      // RHF needs to know the form is "clean" after save
      onSuccess: (data) => form.reset(data),
    })
  }

  // Plug into the Global Bar
  useFormPersistence(form, onSubmit, updateMutation.isPending)

  useEffect(() => {
    if (realm)
      form.reset({
        name: realm.name,
      })
  }, [realm, form])

  useEffect(() => {
    if (realm?.registration_flow_id) {
      registrationFlowIdRef.current = realm.registration_flow_id
    }
  }, [realm?.registration_flow_id])

  if (isLoading) return null

  const registrationEnabled = Boolean(realm?.registration_flow_id)
  const handleRegistrationToggle = (enabled: boolean) => {
    if (!realm) return
    const flowId = registrationFlowIdRef.current

    if (enabled && !flowId) {
      toast.error('No registration flow is configured for this realm.')
      return
    }

    toggleMutation.mutate({
      registration_flow_id: enabled ? flowId : null,
    })
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Card>
          <CardHeader>
            <CardTitle>Basic Settings</CardTitle>
            <CardDescription>The fundamental identity of your realm.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-6">
              <div id="realm-name" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Realm Name"
                  description="This appears in the URL. Changing this will redirect you."
                  placeholder="e.g. my-tenant"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <div id="realm-registration" className="scroll-mt-24 rounded-md -m-2 p-2">
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Registration</CardTitle>
              <CardDescription>Control whether self-service user registration is active.</CardDescription>
            </CardHeader>
            <CardContent className="flex items-center justify-between">
              <div className="space-y-1">
                <div className="text-sm font-medium">Enable User Registration</div>
                <div className="text-xs text-muted-foreground">
                  Turn off to disable the registration flow for this realm.
                </div>
              </div>
              <Switch
                checked={registrationEnabled}
                onCheckedChange={handleRegistrationToggle}
                aria-label="Enable user registration"
                disabled={toggleMutation.isPending}
              />
            </CardContent>
          </Card>
        </div>
      </form>
    </Form>
  )
}
