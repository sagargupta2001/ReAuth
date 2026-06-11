import { useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/dialog'
import { useAddUserEmail } from '@/features/user/api/useUserEmails.ts'
import { ApiError } from '@/shared/api/client'
import { Checkbox } from '@/shared/ui/checkbox'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'

const addEmailFormSchema = z.object({
  email: z
    .string()
    .trim()
    .min(1, { message: 'Email is required' })
    .email({ message: 'Invalid email address' }),
  is_primary: z.boolean(),
  is_verified: z.boolean(),
})

type AddEmailFormValues = z.infer<typeof addEmailFormSchema>

interface AddEmailDialogProps {
  userId: string
}

export function AddEmailDialog({ userId }: AddEmailDialogProps) {
  const [open, setOpen] = useState(false)
  const mutation = useAddUserEmail(userId)

  const form = useForm<AddEmailFormValues>({
    resolver: zodResolver(addEmailFormSchema),
    defaultValues: { email: '', is_primary: false, is_verified: false },
  })

  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
    if (!newOpen) form.reset()
  }

  const onSubmit = (values: AddEmailFormValues) => {
    mutation.mutate(
      {
        email: values.email.trim(),
        is_primary: values.is_primary,
        is_verified: values.is_verified,
      },
      {
        onSuccess: () => handleOpenChange(false),
        onError: (error) => {
          if (error instanceof ApiError) {
            form.setError('email', { type: 'server', message: error.message })
          }
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button variant="highlight" size="clear" className="items-center gap-1 self-stretch">
          <span className="flex shrink-0 items-center justify-center p-0.5">
            <Plus className="size-4 text-current" strokeWidth={1.5} />
          </span>
          <span className="flex items-center text-sm leading-none font-medium">Add email</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Add email address</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="email"
              label="Email address"
              type="email"
              placeholder="name@example.com"
            />

            <div className="mt-2 flex items-start space-x-3">
              <Checkbox
                id="email_is_primary"
                checked={form.watch('is_primary')}
                onCheckedChange={(checked) => form.setValue('is_primary', checked as boolean)}
              />
              <div className="-mt-0.5 grid gap-1.5 leading-none">
                <label
                  htmlFor="email_is_primary"
                  className="cursor-pointer text-[14px] leading-none font-medium"
                >
                  Set as primary
                </label>
                <p className="text-muted-foreground mt-1 text-[13px]">
                  Promote this address to the account&apos;s primary email immediately.
                </p>
              </div>
            </div>

            <div className="flex items-start space-x-3">
              <Checkbox
                id="email_is_verified"
                checked={form.watch('is_verified')}
                onCheckedChange={(checked) => form.setValue('is_verified', checked as boolean)}
              />
              <div className="-mt-0.5 grid gap-1.5 leading-none">
                <label
                  htmlFor="email_is_verified"
                  className="cursor-pointer text-[14px] leading-none font-medium"
                >
                  Mark as verified
                </label>
                <p className="text-muted-foreground mt-1 text-[13px]">
                  Skip the verification step and treat this address as already confirmed.
                </p>
              </div>
            </div>
          </div>
          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={mutation.isPending}>
              {mutation.isPending ? 'Adding...' : 'Add email'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
