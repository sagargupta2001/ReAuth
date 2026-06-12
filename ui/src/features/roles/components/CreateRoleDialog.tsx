import { useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/dialog'
import { useCreateRole } from '@/features/roles/api/useCreateRole'
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema'
import { ApiError } from '@/shared/api/client'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'
import { Separator } from '@/shared/ui/separator'

interface CreateRoleDialogProps {
  clientId?: string
}

export function CreateRoleDialog({ clientId }: CreateRoleDialogProps) {
  const [open, setOpen] = useState(false)
  const mutation = useCreateRole()

  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
    if (!newOpen) {
      form.reset()
    }
  }

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(
      { ...values, client_id: clientId },
      {
        onSuccess: () => handleOpenChange(false),
        onError: (error) => {
          if (error instanceof ApiError) {
            form.setError('name', {
              type: 'server',
              message: error.message,
            })
          }
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button size="sm" className="flex items-center gap-2">
          <Plus size={18} />
          <span>Create Role</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Create new role</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
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
          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={mutation.isPending}>
              {mutation.isPending ? 'Creating...' : 'Create Role'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
