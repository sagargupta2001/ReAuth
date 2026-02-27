import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useUpdateRealm } from '@/features/realm/api/useUpdateRealm.ts'
import { type TokenSettingsSchema, tokenSettingsSchema } from '@/features/realm/schema/setting.schema.ts'
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

export function TokenSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()

  // Don't initialize mutation until we have the ID
  const updateMutation = useUpdateRealm(realm?.id || '')

  const form = useForm<TokenSettingsSchema>({
    resolver: zodResolver(tokenSettingsSchema) as Resolver<TokenSettingsSchema>,
    defaultValues: {
      access_token_ttl_secs: 900,
      refresh_token_ttl_secs: 604800,
      pkce_required_public_clients: true,
      lockout_threshold: 5,
      lockout_duration_secs: 900,
    },
  })

  // Reset form when data loads
  useEffect(() => {
    if (realm) {
      form.reset({
        access_token_ttl_secs: realm.access_token_ttl_secs,
        refresh_token_ttl_secs: realm.refresh_token_ttl_secs,
        pkce_required_public_clients: realm.pkce_required_public_clients,
        lockout_threshold: realm.lockout_threshold,
        lockout_duration_secs: realm.lockout_duration_secs,
      })
    }
  }, [realm, form])

  const onSubmit = (values: TokenSettingsSchema) => {
    updateMutation.mutate(values, {
      // RHF needs to know the form is "clean" after save
      onSuccess: (data) => form.reset(data),
    })
  }

  // Plug into the Global Bar
  useFormPersistence(form, onSubmit, updateMutation.isPending)

  if (isLoading) return <div>Loading settings...</div>
  if (!realm) return <div>Realm not found</div>

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card>
          <CardHeader>
            <CardTitle>Tokens</CardTitle>
            <CardDescription>
              Manage how long sessions and access tokens remain valid.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div id="token-access-ttl" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="access_token_ttl_secs"
                  label="Access Token Lifespan (Seconds)"
                  description="Usually short-lived (e.g., 900s = 15m)."
                  type="number" // Critical: tells browser to show number controls
                />
              </div>

              <div id="token-refresh-ttl" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="refresh_token_ttl_secs"
                  label="SSO Session Idle (Seconds)"
                  description="How long a user stays logged in (e.g., 604800s = 7d)."
                  type="number" // Critical
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Login Protection</CardTitle>
            <CardDescription>Harden public client auth and slow brute-force attempts.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div id="token-pkce-required" className="scroll-mt-24 rounded-md -m-2 p-2">
              <FormField
                control={form.control}
                name="pkce_required_public_clients"
                render={({ field }) => (
                  <FormItem className="flex items-center justify-between gap-6">
                    <div className="space-y-1">
                      <FormLabel>Require PKCE for Public Clients</FormLabel>
                      <FormDescription>
                        Enforce PKCE for SPAs and mobile apps without client secrets.
                      </FormDescription>
                    </div>
                    <FormControl>
                      <Switch
                        checked={field.value}
                        onCheckedChange={field.onChange}
                        aria-label="Require PKCE for public clients"
                        disabled={updateMutation.isPending}
                      />
                    </FormControl>
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
              <div id="token-lockout-threshold" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="lockout_threshold"
                  label="Lockout Threshold (Failed Attempts)"
                  description="Use 0 to disable lockout protection."
                  type="number"
                />
              </div>

              <div id="token-lockout-duration" className="scroll-mt-24 rounded-md -m-2 p-2">
                <FormInput
                  control={form.control}
                  name="lockout_duration_secs"
                  label="Lockout Duration (Seconds)"
                  description="Length of lockout after reaching the threshold."
                  type="number"
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  )
}
