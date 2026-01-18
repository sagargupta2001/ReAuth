import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import type { OidcClient } from '@/entities/oidc/model/types.ts'
import { useUpdateClient } from '@/features/client/api/useUpdateClient.ts'
import {
  type CreateClientSchema,
  createClientSchema,
} from '@/features/client/schema/create.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { Label } from '@/shared/ui/label.tsx'

import { ClientSecretInput } from './ClientSecretInput'

interface ClientSettingsTabProps {
  client: OidcClient
}

export function ClientSettingsTab({ client }: ClientSettingsTabProps) {
  const { t } = useTranslation('client')
  const mutation = useUpdateClient(client.id)

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema()),
    defaultValues: {
      client_id: client.client_id,
      redirect_uris: [{ value: '' }],
      web_origins: [{ value: '' }],
    },
  })

  const redirectUriFields = useFieldArray({ control: form.control, name: 'redirect_uris' })
  const webOriginFields = useFieldArray({ control: form.control, name: 'web_origins' })

  useEffect(() => {
    try {
      const uris = JSON.parse(client.redirect_uris || '[]') as string[]
      const origins = JSON.parse(client.web_origins || '[]') as string[]

      form.reset({
        client_id: client.client_id,
        redirect_uris: uris.length ? uris.map((u) => ({ value: u })) : [{ value: '' }],
        web_origins: origins.length ? origins.map((u) => ({ value: u })) : [{ value: '' }],
      })
    } catch (e) {
      console.error('Failed to parse client JSON fields', e)
    }
  }, [client, form])

  const onSubmit = (values: CreateClientSchema) => {
    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: values.redirect_uris.map((u) => u.value),
        web_origins: values.web_origins?.map((u) => u.value) || [],
      },
      { onSuccess: () => form.reset(values) },
    )
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  return (
    <div className="max-w-4xl space-y-6 p-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          {/* Section 1: Identity */}
          <Card>
            <CardHeader>
              <CardTitle>Client Identity</CardTitle>
              <CardDescription>Core credentials for OIDC authentication.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <FormInput
                control={form.control}
                name="client_id"
                label="Client ID"
                description="The unique identifier used in your application."
              />
              <ClientSecretInput secret={client.client_secret} />
            </CardContent>
          </Card>

          {/* Section 2: Access & Security */}
          <Card>
            <CardHeader>
              <CardTitle>Access Settings</CardTitle>
              <CardDescription>Configure allowed URLs and CORS policies.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Redirect URIs */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label>Valid Redirect URIs</Label>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => redirectUriFields.append({ value: '' })}
                  >
                    <Plus className="mr-2 h-3.5 w-3.5" /> Add URI
                  </Button>
                </div>
                <div className="space-y-2">
                  {redirectUriFields.fields.map((field, index) => (
                    <div key={field.id} className="flex gap-2">
                      <FormField
                        control={form.control}
                        name={`redirect_uris.${index}.value`}
                        render={({ field }) => (
                          <FormItem className="flex-1 space-y-0">
                            <FormControl>
                              <Input placeholder="https://myapp.com/callback" {...field} />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => redirectUriFields.remove(index)}
                        className="text-muted-foreground hover:text-destructive"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>

                {/* FIX: Use a <p> tag instead of FormDescription here */}
                <p className="text-muted-foreground text-[0.8rem]">
                  {t('FORMS.EDIT_CLIENT.FIELDS.VALID_REDIRECT_URIS_HELPER_TEXT')}
                </p>
              </div>

              {/* Web Origins */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <Label>Web Origins (CORS)</Label>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => webOriginFields.append({ value: '' })}
                  >
                    <Plus className="mr-2 h-3.5 w-3.5" /> Add Origin
                  </Button>
                </div>
                <div className="space-y-2">
                  {webOriginFields.fields.map((field, index) => (
                    <div key={field.id} className="flex gap-2">
                      <FormField
                        control={form.control}
                        name={`web_origins.${index}.value`}
                        render={({ field }) => (
                          <FormItem className="flex-1 space-y-0">
                            <FormControl>
                              <Input placeholder="https://myapp.com" {...field} />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => webOriginFields.remove(index)}
                        className="text-muted-foreground hover:text-destructive"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>
    </div>
  )
}
