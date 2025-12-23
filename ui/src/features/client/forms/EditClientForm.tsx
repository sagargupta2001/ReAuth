import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { Button } from '@/shared/ui/button.tsx'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/shared/ui/form.tsx'
import { Input } from '@/shared/ui/input.tsx'
import { Label } from '@/shared/ui/label.tsx'
import { Separator } from '@/shared/ui/separator.tsx'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'

import { useClient } from '../api/useClient.ts'
import { useUpdateClient } from '../api/useUpdateClient.ts'
import { type CreateClientSchema, createClientSchema } from '../schema/create.schema.ts'
import { ClientSecretInput } from '../components/ClientSecretInput.tsx'

interface Props {
  clientId: string
}

export function EditClientForm({ clientId }: Props) {
  const { t } = useTranslation('client')
  const { data: client, isLoading } = useClient(clientId)
  const mutation = useUpdateClient(clientId)

  const schema = createClientSchema()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(schema),
    defaultValues: {
      client_id: '',
      redirect_uris: [{ value: '' }],
      web_origins: [{ value: '' }], // [NEW] Initialize default
    },
  })

  // 1. Hook for Redirect URIs
  const redirectUriFields = useFieldArray({
    control: form.control,
    name: 'redirect_uris',
  })

  // 2. [NEW] Hook for Web Origins
  const webOriginFields = useFieldArray({
    control: form.control,
    name: 'web_origins',
  })

  // 3. Hydrate form data from API response
  useEffect(() => {
    if (client) {
      try {
        const uris = JSON.parse(client.redirect_uris) as string[]
        // [NEW] Parse web_origins (safe fallback to empty array)
        const origins = client.web_origins ? (JSON.parse(client.web_origins) as string[]) : []

        form.reset({
          client_id: client.client_id,
          redirect_uris: uris.map((u) => ({ value: u })),
          web_origins: origins.length > 0 ? origins.map((u) => ({ value: u })) : [{ value: '' }],
        })
      } catch (e) {
        console.error('Failed to parse client JSON fields', e)
      }
    }
  }, [client, form])

  const onSubmit = (values: CreateClientSchema) => {
    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: values.redirect_uris.map((u) => u.value),
        web_origins: values.web_origins ? values.web_origins.map((u) => u.value) : [], // [NEW] Map to string[]
      },
      {
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
        <h3 className="text-lg font-medium">{t('FORMS.EDIT_CLIENT.TITLE')} </h3>
        <p className="text-muted-foreground text-sm">{t('FORMS.EDIT_CLIENT.DESCRIPTION')}</p>
      </div>

      <Separator />
      <Form {...form}>
        <div className="space-y-8">
          <div className="bg-muted/30 grid gap-6 rounded-lg border p-4">
            <div className="grid gap-2">
              <FormInput
                control={form.control}
                name="client_id"
                label={t('FORMS.EDIT_CLIENT.FIELDS.CLIENT_ID')}
                description="Unique identifier for this client."
              />
            </div>
            <ClientSecretInput secret={client?.client_secret} />
          </div>

          <Separator />

          {/* REDIRECT URIS SECTION */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label className="text-base">
                {t('FORMS.EDIT_CLIENT.FIELDS.VALID_REDIRECT_URIS')}
              </Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => redirectUriFields.append({ value: '' })}
              >
                <Plus className="mr-2 h-3.5 w-3.5" /> {t('FORMS.EDIT_CLIENT.ADD_URI_BTN')}
              </Button>
            </div>

            <p className="text-muted-foreground text-sm">
              {t('FORMS.EDIT_CLIENT.FIELDS.VALID_REDIRECT_URIS_HELPER_TEXT')}
            </p>

            <div className="space-y-3">
              {redirectUriFields.fields.map((field, index) => (
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
                          onClick={() => redirectUriFields.remove(index)}
                          disabled={redirectUriFields.fields.length === 1 && index === 0}
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

          <Separator />

          {/* [NEW] WEB ORIGINS SECTION */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label className="text-base">Web Origins (CORS)</Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => webOriginFields.append({ value: '' })}
              >
                <Plus className="mr-2 h-3.5 w-3.5" /> Add Origin
              </Button>
            </div>

            <p className="text-muted-foreground text-sm">
              Allowed origins for Cross-Origin Resource Sharing (CORS). Enter URL origins (e.g.,
              http://localhost:6565).
            </p>

            <div className="space-y-3">
              {webOriginFields.fields.map((field, index) => (
                <FormField
                  key={field.id}
                  control={form.control}
                  name={`web_origins.${index}.value`}
                  render={({ field }) => (
                    <FormItem>
                      <div className="flex items-center gap-2">
                        <FormControl>
                          <Input placeholder="http://localhost:6565" {...field} />
                        </FormControl>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          onClick={() => webOriginFields.remove(index)}
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
