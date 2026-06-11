import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'

import { useUpdateUser } from '@/features/user/api/useUpdateUser.ts'
import { useUser } from '@/features/user/api/useUser.ts'

const usernameSchema = z.object({
  username: z.string().min(3, 'Username must be at least 3 characters'),
  first_name: z.string(),
  last_name: z.string(),
})

type ProfileFormValues = z.infer<typeof usernameSchema>

interface ProfileSectionProps {
  userId: string
}

export function ProfileSection({ userId }: ProfileSectionProps) {
  const { data: user, isLoading } = useUser(userId)
  const mutation = useUpdateUser(userId)

  const form = useForm<ProfileFormValues>({
    resolver: zodResolver(usernameSchema),
    defaultValues: { username: '', first_name: '', last_name: '' },
  })

  const onSubmit = (values: ProfileFormValues) => {
    const payload = {
      username: values.username.trim(),
      first_name: values.first_name.trim() || null,
      last_name: values.last_name.trim() || null,
    }

    mutation.mutate(payload, {
      onSuccess: () =>
        form.reset({
          username: payload.username,
          first_name: payload.first_name ?? '',
          last_name: payload.last_name ?? '',
        }),
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  useEffect(() => {
    if (user) {
      form.reset({
        username: user.username,
        first_name: user.first_name ?? '',
        last_name: user.last_name ?? '',
      })
    }
  }, [user, form])

  if (isLoading)
    return (
      <Card>
        <CardContent className="space-y-3 pt-6">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-10" />
          <Skeleton className="h-10" />
          <Skeleton className="h-10" />
        </CardContent>
      </Card>
    )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Personal Information</CardTitle>
      </CardHeader>
      <CardContent>
        <div className='bg-primary-foreground p-4 rounded-2xl'>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormInput control={form.control} name="username" label="Username" />
              <div className="grid gap-4 sm:grid-cols-2">
                <FormInput control={form.control} name="first_name" label="First name" />
                <FormInput control={form.control} name="last_name" label="Last name" />
              </div>
            </form>
          </Form>
        </div>
      </CardContent>
    </Card>
  )
}
