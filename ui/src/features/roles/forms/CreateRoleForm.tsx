import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { DialogFooter } from '@/components/dialog'
import { Form } from '@/components/form'
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea.tsx'
import { useCreateRole } from '@/features/roles/api/useCreateRole.ts'



interface CreateRoleFormProps {
  clientId?: string
  onSuccess?: () => void
  onCancel?: () => void
  isDialog?: boolean
}

export function CreateRoleForm({
  clientId,
  onSuccess,
  onCancel,
  isDialog = false,
}: CreateRoleFormProps) {
  const mutation = useCreateRole()

  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(
      { ...values, client_id: clientId },
      {
        onSuccess: () => {
          form.reset()
          onSuccess?.()
        },
      },
    )
  }

  // Only enable Persistence (Floating Bar) if NOT in a dialog
  useFormPersistence(form, onSubmit, mutation.isPending, { enabled: !isDialog })

  return (
    <div className={isDialog ? '' : 'max-w-2xl space-y-8'}>
      {!isDialog && (
        <div>
          <h3 className="text-lg font-medium">Create Role</h3>
          <p className="text-muted-foreground text-sm">
            Define a new role. You can assign permissions after creation.
          </p>
        </div>
      )}

      <Form {...form}>
        {/* We need a real <form> tag to handle standard submits in Dialogs */}
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
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

          {/* Conditional Footer for Dialog Mode */}
          {isDialog && (
            <DialogFooter className="mt-6">
              <Button type="button" variant="outline" onClick={onCancel}>
                Cancel
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                {mutation.isPending ? 'Creating...' : 'Create Role'}
              </Button>
            </DialogFooter>
          )}
        </form>
      </Form>
    </div>
  )
}
