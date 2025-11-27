import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { type Resolver, useForm } from 'react-hook-form'

import { useCurrentRealm } from '@/entities/realm/api/useRealm.ts'
import { useUpdateRealm } from '@/entities/realm/api/useUpdateRealm.ts'
import {
  type GeneralSettingsSchema,
  generalSettingsSchema,
} from '@/features/realm/settings/schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form } from '@/shared/ui/form.tsx'

export function GeneralSettingsForm() {
  const { data: realm, isLoading } = useCurrentRealm()
  const updateMutation = useUpdateRealm(realm?.id || '')

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
        </Card>
      </form>
    </Form>
  )
}
