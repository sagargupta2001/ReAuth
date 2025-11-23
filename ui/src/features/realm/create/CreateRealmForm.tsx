import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useNavigate } from 'react-router-dom'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/card'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { useCreateRealm } from '@/entities/realm/api/useCreateRealm.ts'

const formSchema = z.object({
  name: z
    .string()
    .min(3, { message: 'Realm name must be at least 3 characters.' })
    .max(30, { message: 'Realm name must be less than 30 characters.' })
    .regex(/^[a-z0-9-]+$/, {
      message: 'Only lowercase letters, numbers, and hyphens allowed.',
    }),
})

type FormValues = z.infer<typeof formSchema>

export function CreateRealmForm() {
  const navigate = useNavigate()
  const createRealmMutation = useCreateRealm()

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
    },
  })

  const onSubmit = (values: FormValues) => {
    createRealmMutation.mutate(values)
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>Create Realm</CardTitle>
        <CardDescription>
          A realm is a fully isolated environment. Create a new one to manage its own users, roles,
          and applications.
        </CardDescription>
      </CardHeader>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)}>
          <CardContent className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Realm Name</FormLabel>
                  <FormControl>
                    <Input placeholder="e.g., my-realm" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
          <CardFooter className="flex justify-between">
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate(-1)} // Go back
              disabled={createRealmMutation.isPending}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={createRealmMutation.isPending}>
              {createRealmMutation.isPending ? 'Creating...' : 'Create Realm'}
            </Button>
          </CardFooter>
        </form>
      </Form>
    </Card>
  )
}
