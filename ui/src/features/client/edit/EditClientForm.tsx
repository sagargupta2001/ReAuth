import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Separator } from '@/components/separator'
import { Skeleton } from '@/components/skeleton'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input.tsx'

import { useClient } from '../api/useClient'
import { useUpdateClient } from '../api/useUpdateClient'
import { type CreateClientSchema, createClientSchema } from '../create/schema'
import { ClientSecretInput } from './components/ClientSecretInput'

interface Props {
  clientId: string
}

export function EditClientForm({ clientId }: Props) {
  const { data: client, isLoading } = useClient(clientId)
  const mutation = useUpdateClient(clientId)

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema),
    defaultValues: { client_id: '', redirect_uris: [{ value: '' }] },
  })

  useEffect(() => {
    if (client) {
      try {
        const uris = JSON.parse(client.redirect_uris) as string[]
        form.reset({
          client_id: client.client_id,
          redirect_uris: uris.map((u) => ({ value: u })),
        })
      } catch {
        /* ignore json error */
      }
    }
  }, [client, form])

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: 'redirect_uris',
  })

  const onSubmit = (values: CreateClientSchema) => {
    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: values.redirect_uris.map((u) => u.value),
      },
      {
        // On success, we reset the form with the *new* values so the bar disappears
        onSuccess: () => form.reset(values),
      },
    )
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  if (isLoading)
    return (
      <div className="space-y-4">
        <Skeleton className="h-12" />
        <Skeleton className="h-48" />
      </div>
    )

  return (
    <div className="max-w-2xl space-y-8">
      <div>
        <h3 className="text-lg font-medium">Create OIDC Client</h3>
        <p className="text-muted-foreground text-sm">
          Register a new application that can authenticate with this realm.
        </p>
      </div>

      <Separator />
      <Form {...form}>
        <div className="space-y-8">
          <div className="bg-muted/30 grid gap-6 rounded-lg border p-4">
            <div className="grid gap-2">
              <FormInput
                control={form.control}
                name="client_id"
                label="Client ID"
                description="Unique identifier for this client."
              />
            </div>
            <ClientSecretInput secret={client?.client_secret} />
          </div>

          <Separator />

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label className="text-base">Valid Redirect URIs</Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => append({ value: '' })}
              >
                <Plus className="mr-2 h-3.5 w-3.5" /> Add URI
              </Button>
            </div>

            <p className="text-muted-foreground text-sm">
              After login, ReAuth can only redirect to these specific URLs.
            </p>

            <div className="space-y-3">
              {fields.map((field, index) => (
                <FormField
                  key={field.id}
                  control={form.control}
                  name={`redirect_uris.${index}.value`}
                  render={({ field }) => (
                    <FormItem>
                      <div className="flex items-center gap-2">
                        <FormControl>
                          <Input {...field} />
                        </FormControl>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          onClick={() => remove(index)}
                          disabled={fields.length === 1 && index === 0}
                          className="text-muted-foreground hover:text-destructive shrink-0"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              ))}
            </div>
          </div>
        </div>
      </Form>
    </div>
  )
}
