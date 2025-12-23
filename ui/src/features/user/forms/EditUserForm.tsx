import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Separator } from '@radix-ui/react-select'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Form } from '@/shared/ui/form.tsx'
import { useUpdateUser } from '@/features/user/api/useUpdateUser.ts'
import { useUser } from '@/features/user/api/useUser.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

const formSchema = z.object({
  username: z.string().min(3, 'Username must be at least 3 characters'),
})

type FormValues = z.infer<typeof formSchema>

export function EditUserForm({ userId }: { userId: string }) {
  const { data: user, isLoading } = useUser(userId)
  const mutation = useUpdateUser(userId)

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: { username: '' },
  })

  useEffect(() => {
    if (user) {
      form.reset({ username: user.username })
    }
  }, [user, form])

  const onSubmit = (values: FormValues) => {
    mutation.mutate(values, {
      onSuccess: () => form.reset({ username: values.username }), // Reset dirty state
    })
  }

  // Use Floating Action Bar for edits
  useFormPersistence(form, onSubmit, mutation.isPending)

  if (isLoading)
    return (
      <div className="space-y-4">
        <Skeleton className="h-12" />
        <Skeleton className="h-24" />
      </div>
    )

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h3 className="text-lg font-medium">Edit User</h3>
        <p className="text-muted-foreground text-sm">Update user details.</p>
      </div>
      <Separator />

      <Form {...form}>
        <div className="space-y-8">
          <div className="bg-muted/30 grid gap-4 rounded-lg border p-4">
            <FormInput control={form.control} name="username" label="Username" />
          </div>
          {/* Password reset logic would go here in a separate section/form */}
        </div>
      </Form>
    </div>
  )
}
