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
import { useCurrentRealm } from '@/entities/realm/api/useRealm'
import { useUpdateRealm } from '@/entities/realm/api/useUpdateRealm.ts'
import {
  type GeneralSettingsSchema,
  generalSettingsSchema,
} from '@/features/realm/settings/schema.ts'
import { FormInput } from '@/shared/ui/form-input'

export function GeneralSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()
  const updateMutation = useUpdateRealm(realm?.id || '')

  const form = useForm<GeneralSettingsSchema>({
    resolver: zodResolver(generalSettingsSchema) as Resolver<GeneralSettingsSchema>,
    defaultValues: {
      name: '',
    },
  })

  useEffect(() => {
    if (realm) {
      form.reset({
        name: realm.name,
      })
    }
  }, [realm, form])

  const onSubmit = (values: GeneralSettingsSchema) => {
    updateMutation.mutate(values)
  }

  if (isLoading) return null

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
              <FormInput
                control={form.control}
                name="name"
                label="Realm Name"
                description="This appears in the URL. Changing this will redirect you."
                placeholder="e.g. my-tenant"
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
              Save
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  )
}
