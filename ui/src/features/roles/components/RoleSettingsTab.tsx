import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Form } from '@/components/form'
import type { Role } from '@/features/roles/api/useRoles.ts'
import { useUpdateRole } from '@/features/roles/api/useUpdateRole'
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'

interface RoleSettingsTabProps {
  role: Role
}

export function RoleSettingsTab({ role }: RoleSettingsTabProps) {
  const mutation = useUpdateRole(role.id)

  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: role.name,
      description: role.description || '',
    },
  })

  // Sync form with data when role loads/changes
  useEffect(() => {
    form.reset({
      name: role.name,
      description: role.description || '',
    })
  }, [role, form])

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => {
        // Reset with new values to mark form as pristine (clean)
        form.reset(values)
      },
    })
  }

  // Enable Floating Save Bar
  useFormPersistence(form, onSubmit, mutation.isPending)

  return (
    <div className="max-w-4xl space-y-6 p-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>General Settings</CardTitle>
              <CardDescription>
                Manage the basic identification details for this role.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Role Name"
                  placeholder="e.g. content_manager"
                  description="The unique identifier used in code checks. Changing this may break existing authorization logic."
                />

                <FormTextarea
                  control={form.control}
                  name="description"
                  label="Description"
                  placeholder="Describe the purpose and responsibilities of this role..."
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
