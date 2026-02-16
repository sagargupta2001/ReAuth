import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { DialogFooter } from '@/components/dialog'
import { Form } from '@/components/form'
import { useCreateGroup } from '@/features/group/api/useCreateGroup'
import { type GroupFormValues, groupSchema } from '@/features/group/schema/create.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'

interface CreateGroupFormProps {
  onSuccess?: () => void
  onCancel?: () => void
  isDialog?: boolean
}

export function CreateGroupForm({
  onSuccess,
  onCancel,
  isDialog = false,
}: CreateGroupFormProps) {
  const mutation = useCreateGroup()

  const form = useForm<GroupFormValues>({
    resolver: zodResolver(groupSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const onSubmit = (values: GroupFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => {
        form.reset()
        onSuccess?.()
      },
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending, { enabled: !isDialog })

  return (
    <div className={isDialog ? '' : 'max-w-2xl space-y-8'}>
      {!isDialog && (
        <div>
          <h3 className="text-lg font-medium">Create Group</h3>
          <p className="text-muted-foreground text-sm">
            Groups collect users and roles for easier assignment.
          </p>
        </div>
      )}

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <div className="grid gap-4">
            <FormInput
              control={form.control}
              name="name"
              label="Group Name"
              placeholder="e.g. product-team"
              description="A human-friendly identifier for this group."
            />
            <FormTextarea
              control={form.control}
              name="description"
              label="Description"
              placeholder="Describe who belongs to this group..."
            />
          </div>

          {isDialog && (
            <DialogFooter className="mt-6">
              <Button type="button" variant="outline" onClick={onCancel}>
                Cancel
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                {mutation.isPending ? 'Creating...' : 'Create Group'}
              </Button>
            </DialogFooter>
          )}
        </form>
      </Form>
    </div>
  )
}
