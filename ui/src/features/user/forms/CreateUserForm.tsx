import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Form } from '@/shared/ui/form.tsx'
import { useCreateUser } from '@/features/user/api/useCreateUser.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'

const formSchema = z.object({
  username: z.string().min(3),
  password: z.string().min(8),
})

export function CreateUserForm() {
  const mutation = useCreateUser()
  const form = useForm({
    resolver: zodResolver(formSchema),
    defaultValues: { username: '', password: '' },
  })

  const onSubmit = (values: any) => mutation.mutate(values, { onSuccess: () => form.reset() })

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
          <FormInput control={form.control} name="password" label="Password" type="password" />
        </div>
      </Form>
    </div>
  )
}
