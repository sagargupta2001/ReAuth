import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, Save } from 'lucide-react'
import { type Resolver, useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/card'
import { Form } from '@/components/form'
import { useCurrentRealm } from '@/entities/realm/api/useRealm.ts'
import { useUpdateRealm } from '@/entities/realm/api/useUpdateRealm.ts'
import { type TokenSettingsSchema, tokenSettingsSchema } from '@/features/realm/settings/schema.ts'
import { FormInput } from '@/shared/ui/form-input'

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
    updateMutation.mutate(values)
  }

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
              <FormInput
                control={form.control}
                name="access_token_ttl_secs"
                label="Access Token Lifespan (Seconds)"
                description="Usually short-lived (e.g., 900s = 15m)."
                type="number" // Critical: tells browser to show number controls
              />

              <FormInput
                control={form.control}
                name="refresh_token_ttl_secs"
                label="SSO Session Idle (Seconds)"
                description="How long a user stays logged in (e.g., 604800s = 7d)."
                type="number" // Critical
              />
            </div>
          </CardContent>
          <CardFooter className="border-t px-6 py-4">
            <Button type="submit" disabled={updateMutation.isPending}>
              {updateMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              Save Changes
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  )
}
