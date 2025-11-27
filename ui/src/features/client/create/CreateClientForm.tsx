import { zodResolver } from '@hookform/resolvers/zod'
import { Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { useCreateClient } from '@/features/client/api/useCreateClient.ts'
import { type CreateClientSchema, createClientSchema } from '@/features/client/create/schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence.ts'
import { Button } from '@/shared/ui/button.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/shared/ui/form.tsx'
import { Input } from '@/shared/ui/input.tsx'
import { Separator } from '@/shared/ui/separator.tsx'

export function CreateClientForm() {
  const { t } = useTranslation('client')
  const mutation = useCreateClient()

  const schema = createClientSchema()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(schema),
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
        <h3 className="text-lg font-medium">{t('FORMS.CREATE_CLIENT.TITLE')}</h3>
        <p className="text-muted-foreground text-sm">{t('FORMS.CREATE_CLIENT.DESCRIPTION')}</p>
      </div>

      <Separator />

      <Form {...form}>
        <div className="grid gap-4">
          <FormInput
            control={form.control}
            name="client_id"
            label={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID')}
            placeholder={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID_PLACEHOLDER')}
            description={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID_HELPER_TEXT')}
          />
        </div>

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <label className="text-base leading-none font-medium peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              {t('FORMS.CREATE_CLIENT.FIELDS.VALID_REDIRECT_URIS')}
            </label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => append({ value: '' })}
              className="h-8"
            >
              <Plus className="mr-2 h-3.5 w-3.5" />
              {t('FORMS.CREATE_CLIENT.ADD_URI_BTN')}
            </Button>
          </div>

          <p className="text-muted-foreground text-sm">
            {t('FORMS.CREATE_CLIENT.FIELDS.VALID_REDIRECT_URIS_HELPER_TEXT')}
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
                        <Input
                          placeholder={t(
                            'FORMS.CREATE_CLIENT.FIELDS.VALID_REDIRECT_URIS_PLACEHOLDER',
                          )}
                          {...field}
                        />
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
