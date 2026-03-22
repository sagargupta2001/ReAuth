import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Form } from '@/shared/ui/form.tsx'
import { useCreateUser } from '@/features/user/api/useCreateUser.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'

const emailSchema = z
  .string()
  .trim()
  .email('Enter a valid email')
  .or(z.literal(''))
  .optional()

const formSchema = z.object({
  username: z.string().min(3),
  email: emailSchema,
  password: z.string().min(8),
})

type FormValues = z.infer<typeof formSchema>

export function CreateUserForm() {
  const mutation = useCreateUser()
  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: { username: '', email: '', password: '' },
  })

  const onSubmit = (values: FormValues) => {
    const email = values.email?.trim() || undefined
    mutation.mutate({ ...values, email }, { onSuccess: () => form.reset() })
  }

  // Floating Bar
  useFormPersistence(form, onSubmit, mutation.isPending)

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h3 className="text-lg font-medium">Create User</h3>
        <p className="text-muted-foreground text-sm">Add a new user to the system.</p>
      </div>
      <Form {...form}>
        <div className="grid gap-4">
          <FormInput control={form.control} name="username" label="Username" />
          <FormInput control={form.control} name="email" label="Email" type="email" />
          <FormInput control={form.control} name="password" label="Password" type="password" />
        </div>
      </Form>
    </div>
  )
}
