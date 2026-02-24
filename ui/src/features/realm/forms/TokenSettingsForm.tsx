import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/features/realm/api/useRealm.ts'
import { useUpdateRealm } from '@/features/realm/api/useUpdateRealm.ts'
import { type TokenSettingsSchema, tokenSettingsSchema } from '@/features/realm/schema/setting.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'

export function TokenSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()

  // Don't initialize mutation until we have the ID
  const updateMutation = useUpdateRealm(realm?.id || '')

  const form = useForm<TokenSettingsSchema>({
    resolver: zodResolver(tokenSettingsSchema) as Resolver<TokenSettingsSchema>,
    defaultValues: {
      access_token_ttl_secs: 900,
      refresh_token_ttl_secs: 604800,
    },
  })

  // Reset form when data loads
  useEffect(() => {
    if (realm) {
      form.reset({
        access_token_ttl_secs: realm.access_token_ttl_secs,
        refresh_token_ttl_secs: realm.refresh_token_ttl_secs,
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
      </form>
    </Form>
  )
}
