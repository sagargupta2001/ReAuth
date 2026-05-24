import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Mail } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { useUpdateUser } from '@/features/user/api/useUpdateUser.ts'
import { useUser } from '@/features/user/api/useUser.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card.tsx'
import { Form } from '@/shared/ui/form.tsx'
import { Skeleton } from '@/shared/ui/skeleton.tsx'

const emailSchema = z.string().trim().email('Enter a valid email').or(z.literal('')).optional()

const formSchema = z.object({
  username: z.string().min(3, 'Username must be at least 3 characters'),
  email: emailSchema,
})

type FormValues = z.infer<typeof formSchema>

export function UseProfileTab({ userId }: { userId: string }) {
  const { data: user, isLoading } = useUser(userId)
  const mutation = useUpdateUser(userId)

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: { username: '', email: '' },
  })

  const onSubmit = (values: FormValues) => {
    const email = values.email?.trim() || undefined
    mutation.mutate(
      { ...values, email },
      {
        onSuccess: () => form.reset({ username: values.username, email: email ?? '' }),
      },
    )
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  useEffect(() => {
    if (user) {
      form.reset({ username: user.username, email: user.email ?? '' })
    }
  }, [user, form])

  if (isLoading)
    return (
      <div className="space-y-4">
        <Skeleton className="h-12" />
        <Skeleton className="h-24" />
      </div>
    )

  return (
    <Form {...form}>
      <Card className="w-full">
        <CardHeader>
          <CardTitle>Email addresses</CardTitle>
        </CardHeader>

        <CardContent className="grid gap-2">
          <div className="flex items-center justify-between rounded-2xl bg-black p-4 text-white">
            <div className="flex items-center gap-3">
              <Mail className="h-4 w-4 text-white" />
              <div className="flex flex-col">
                <span className="text-sm font-medium">{user?.email}</span>
              </div>
            </div>
            <span className="text-xs text-zinc-400">added almost 2y ago</span>
          </div>
        </CardContent>
      </Card>
    </Form>
  )
}
