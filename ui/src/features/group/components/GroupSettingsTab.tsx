import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Form } from '@/components/form'
import type { Group } from '@/entities/group/model/types'
import { useUpdateGroup } from '@/features/group/api/useUpdateGroup'
import { type GroupFormValues, groupSchema } from '@/features/group/schema/create.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'

interface GroupSettingsTabProps {
  group: Group
}

export function GroupSettingsTab({ group }: GroupSettingsTabProps) {
  const mutation = useUpdateGroup(group.id)

  const form = useForm<GroupFormValues>({
    resolver: zodResolver(groupSchema),
    defaultValues: {
      name: group.name,
      description: group.description || '',
    },
  })

  useEffect(() => {
    form.reset({
      name: group.name,
      description: group.description || '',
    })
  }, [group, form])

  const onSubmit = (values: GroupFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => {
        form.reset(values)
      },
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  return (
    <div className="max-w-4xl space-y-6 p-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>General Settings</CardTitle>
              <CardDescription>Manage the basic identification details for this group.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Group Name"
                  placeholder="e.g. product-team"
                  description="The display name shown in the admin UI."
                />

                <FormTextarea
                  control={form.control}
                  name="description"
                  label="Description"
                  placeholder="Describe who should belong to this group..."
                  className="min-h-[120px]"
                />
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>
    </div>
  )
}
