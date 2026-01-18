import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Form } from '@/components/form'
import { Separator } from '@/components/separator'
import { Skeleton } from '@/components/skeleton'
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema.ts'
import { FormInput } from '@/shared/ui/form-input'

import { useRole } from '../api/useRole'
import { FormTextarea } from '@/shared/ui/form-textarea.tsx'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { useUpdateRole } from '@/features/roles/api/useUpdateRole.ts'

export function EditRoleForm({ roleId }: { roleId: string }) {
  const { data: role, isLoading } = useRole(roleId)
  const mutation = useUpdateRole(roleId)

  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: { name: '', description: '' },
  })

  useEffect(() => {
    if (role) {
      form.reset({
        name: role.name,
        description: role.description || '',
      })
    }
  }, [role, form])

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(values, { onSuccess: () => form.reset(values) })
    console.log('Update not implemented yet', values)
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  if (isLoading) {
    return (
      <div className="max-w-2xl space-y-4">
        <Skeleton className="h-12 w-full" />
        <Skeleton className="h-24 w-full" />
      </div>
    )
  }

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h3 className="text-lg font-medium">Edit Role</h3>
        <p className="text-muted-foreground text-sm">Update role details.</p>
      </div>
      <Separator />

      <Form {...form}>
        <div className="space-y-6">
          <div className="bg-muted/30 grid gap-4 rounded-lg border p-4">
            {/* Name is usually immutable in many RBAC systems, but editable here if supported */}
            <FormInput
              control={form.control}
              name="name"
              label="Role Name"
              description="Role name cannot be changed."
            />
            <FormTextarea control={form.control} name="description" label="Description" />
          </div>

          {/* TODO: Add Permissions Picker Component Here in Next Step */}
          <div className="rounded-lg border p-4">
            <h4 className="mb-4 font-medium">Permissions</h4>
            <div className="text-muted-foreground text-sm">
              Permission assignment interface coming soon...
            </div>
          </div>
        </div>
      </Form>
    </div>
  )
}
