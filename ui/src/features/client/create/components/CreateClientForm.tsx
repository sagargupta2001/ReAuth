import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, Save } from 'lucide-react'
import { useForm } from 'react-hook-form'

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
import { Textarea } from '@/components/textarea'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useCreateClient } from '@/features/client/api/useCreateClient.ts'
import { type CreateClientSchema, createClientSchema } from '@/features/client/create/schema.ts'
import { FormInput } from '@/shared/ui/form-input'

export function CreateClientForm() {
  const navigate = useRealmNavigate()
  const mutation = useCreateClient()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema),
    defaultValues: {
      client_id: '',
      redirect_uris: '',
    },
  })

  const onSubmit = (values: CreateClientSchema) => {
    // Convert newline-separated string to array
    const uriArray = values.redirect_uris
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean)

    mutation.mutate({
      client_id: values.client_id,
      redirect_uris: uriArray,
    })
  }

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Create OIDC Client</CardTitle>
        <CardDescription>
          Register a new application that can authenticate with this realm.
        </CardDescription>
      </CardHeader>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)}>
          <CardContent className="space-y-4">
            <FormInput
              control={form.control}
              name="client_id"
              label="Client ID"
              placeholder="e.g. my-react-app"
              description="The unique identifier for your application."
            />

            <FormField
              control={form.control}
              name="redirect_uris"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Valid Redirect URIs</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder={'http://localhost:3000/callback\nhttps://myapp.com/callback'}
                      className="min-h-[100px] font-mono text-sm"
                      {...field}
                    />
                  </FormControl>
                  <p className="text-muted-foreground text-[0.8rem]">
                    Enter one URI per line. These are the allowed callback URLs.
                  </p>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
          <CardFooter className="flex justify-between border-t px-6 py-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate('/clients')}
              disabled={mutation.isPending}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              Create Client
            </Button>
          </CardFooter>
        </form>
      </Form>
    </Card>
  )
}
