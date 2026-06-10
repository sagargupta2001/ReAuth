import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'

import { useUpdateUser } from '@/features/user/api/useUpdateUser.ts'
import { useUser } from '@/features/user/api/useUser.ts'

const usernameSchema = z.object({
  username: z.string().min(3, 'Username must be at least 3 characters'),
})

type UsernameFormValues = z.infer<typeof usernameSchema>

interface ProfileSectionProps {
  userId: string
}

export function ProfileSection({ userId }: ProfileSectionProps) {
  const { data: user, isLoading } = useUser(userId)
  const mutation = useUpdateUser(userId)

  const form = useForm<UsernameFormValues>({
    resolver: zodResolver(usernameSchema),
    defaultValues: { username: '' },
  })

  const onSubmit = (values: UsernameFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => form.reset({ username: values.username }),
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  useEffect(() => {
    if (user) form.reset({ username: user.username })
  }, [user, form])

  if (isLoading)
    return (
      <Card>
        <CardContent className="space-y-3 pt-6">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-10" />
        </CardContent>
      </Card>
    )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Profile</CardTitle>
        <CardDescription>Update this user&apos;s display name.</CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormInput control={form.control} name="username" label="Username" />
          </form>
        </Form>
      </CardContent>
    </Card>
  )
}
