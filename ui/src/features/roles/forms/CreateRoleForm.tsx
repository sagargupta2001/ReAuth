import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';



import { Form } from '@/components/form';
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema.ts';
import { useFormPersistence } from '@/shared/hooks/useFormPersistence';
import { FormInput } from '@/shared/ui/form-input'

import { useCreateRole } from '../api/useCreateRole'
import { FormTextarea } from '@/shared/ui/form-textarea.tsx'
















export function CreateRoleForm() {
  const mutation = useCreateRole()
  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => form.reset(),
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h3 className="text-lg font-medium">Create Role</h3>
        <p className="text-muted-foreground text-sm">
          Define a new role. You can assign permissions after creation.
        </p>
      </div>
      <Form {...form}>
        <div className="grid gap-4">
          <FormInput
            control={form.control}
            name="name"
            label="Role Name"
            placeholder="e.g. content_editor"
            description="Unique identifier. Lowercase, numbers, and underscores only."
          />
          <FormTextarea
            control={form.control}
            name="description"
            label="Description"
            placeholder="Describe the purpose of this role..."
          />
        </div>
      </Form>
    </div>
  )
}
