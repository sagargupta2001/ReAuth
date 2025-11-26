import { zodResolver } from '@hookform/resolvers/zod'
import { Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import { Separator } from '@/components/separator'
import { useCreateClient } from '@/features/client/api/useCreateClient.ts'
import { type CreateClientSchema, createClientSchema } from '@/features/client/create/schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input'

export function CreateClientForm() {
  const mutation = useCreateClient()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema),
    defaultValues: {
      client_id: '',
      redirect_uris: [{ value: '' }],
    },
  })

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: 'redirect_uris',
  })

  const onSubmit = (values: CreateClientSchema) => {
    const flatUris = values.redirect_uris.map((u) => u.value)

    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: flatUris,
      },
      {
        onSuccess: () => {
          form.reset()
        },
      },
    )
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

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
        {/* --- SECTION 1: Basic Info --- */}
        <div className="grid gap-4">
          <FormInput
            control={form.control}
            name="client_id"
            label="Client ID"
            placeholder="e.g. my-react-app"
            description="The unique identifier for your application. Only lowercase letters, numbers, and hyphens."
          />
        </div>

        {/* --- SECTION 2: Redirect URIs --- */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <label className="text-base leading-none font-medium peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Valid Redirect URIs
            </label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => append({ value: '' })}
              className="h-8"
            >
              <Plus className="mr-2 h-3.5 w-3.5" />
              Add URI
            </Button>
          </div>

          <p className="text-muted-foreground text-sm">
            After login, ReAuth can only redirect users to these specific URLs.
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
                        <Input placeholder="https://myapp.com/callback" {...field} />
                      </FormControl>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="text-muted-foreground hover:text-destructive shrink-0"
                        onClick={() => remove(index)}
                        // Prevent removing the last item if you want to enforce at least one
                        disabled={fields.length === 1 && index === 0}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                    <FormMessage />
                  </FormItem>
                )}
              />
            ))}
            {/* Show global array error */}
            {form.formState.errors.redirect_uris?.root && (
              <p className="text-destructive text-sm font-medium">
                {form.formState.errors.redirect_uris.root.message}
              </p>
            )}
          </div>
        </div>
      </Form>
    </div>
  )
}
