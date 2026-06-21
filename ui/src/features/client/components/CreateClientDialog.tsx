import { useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Check, Copy, Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCreateClient } from '@/features/client/api/useCreateClient.ts'
import {
  type CreateClientSchema,
  createClientSchema,
} from '@/features/client/schema/create.schema.ts'
import { ApiError } from '@/shared/api/client'
import { Button } from '@/shared/ui/button.tsx'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/shared/ui/dialog'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/shared/ui/form.tsx'
import { FormInput } from '@/shared/ui/form-input.tsx'
import { Input } from '@/shared/ui/input.tsx'
import { Separator } from '@/shared/ui/separator.tsx'

export function CreateClientDialog() {
  const { t } = useTranslation('client')
  const [open, setOpen] = useState(false)
  const [createdSecret, setCreatedSecret] = useState<string | null>(null)
  const [createdClientId, setCreatedClientId] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const mutation = useCreateClient()
  const navigate = useRealmNavigate()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema()),
    defaultValues: {
      client_id: '',
      redirect_uris: [{ value: '' }],
      web_origins: [{ value: '' }],
    },
  })

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: 'redirect_uris',
  })

  const webOriginFields = useFieldArray({
    control: form.control,
    name: 'web_origins',
  })

  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
    if (!newOpen) {
      form.reset()
    }
  }

  const onSubmit = (values: CreateClientSchema) => {
    const flatUris = values.redirect_uris.map((u) => u.value)
    const webOrigins = values.web_origins?.map((u) => u.value) || []

    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: flatUris,
        web_origins: webOrigins,
      },
      {
        onSuccess: (client) => {
          form.reset()
          setOpen(false)
          setCreatedClientId(client.id)
          setCreatedSecret(client.client_secret ?? null)
        },
        onError: (error) => {
          if (error instanceof ApiError) {
            form.setError('client_id', {
              type: 'server',
              message: error.message,
            })
          }
        },
      },
    )
  }

  return (
    <>
      <Dialog open={open} onOpenChange={handleOpenChange}>
        <DialogTrigger asChild>
          <Button size="sm" className="flex items-center gap-2">
            <Plus size={18} />
            <span>Create Client</span>
          </Button>
        </DialogTrigger>
        <DialogContent className="sm:max-w-[600px]">
          <DialogHeader className="pt-6 pl-6">
            <DialogTitle>{t('FORMS.CREATE_CLIENT.TITLE')}</DialogTitle>
            <DialogDescription>{t('FORMS.CREATE_CLIENT.DESCRIPTION')}</DialogDescription>
          </DialogHeader>

          <Separator className="my-1" />

          <Form {...form}>
            <div className="grid max-h-[60vh] gap-6 overflow-y-auto px-6 pb-2">
              <FormInput
                control={form.control}
                name="client_id"
                label={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID')}
                placeholder={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID_PLACEHOLDER')}
                description={t('FORMS.CREATE_CLIENT.FIELDS.CLIENT_ID_HELPER_TEXT')}
              />

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <label className="text-sm leading-none font-medium">
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

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <label className="text-sm leading-none font-medium">Web Origins (CORS)</label>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => webOriginFields.append({ value: '' })}
                    className="h-8"
                  >
                    <Plus className="mr-2 h-3.5 w-3.5" />
                    Add Origin
                  </Button>
                </div>
                <p className="text-muted-foreground text-sm">
                  Allowed origins for CORS (e.g., http://localhost:6565).
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
                              className="text-muted-foreground hover:text-destructive shrink-0"
                              onClick={() => webOriginFields.remove(index)}
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

            <DialogFooter className="gap-1 py-3 pr-3">
              <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
                Cancel
              </Button>
              <Button
                size="sm"
                onClick={form.handleSubmit(onSubmit)}
                disabled={mutation.isPending}
              >
                {mutation.isPending ? 'Creating...' : 'Create Client'}
              </Button>
            </DialogFooter>
          </Form>
        </DialogContent>
      </Dialog>

      <Dialog
        open={createdSecret != null}
        onOpenChange={(isOpen) => {
          if (!isOpen) {
            setCreatedSecret(null)
            setCopied(false)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Client secret generated</DialogTitle>
            <DialogDescription>
              Copy this secret now. You will not be able to view it again.
            </DialogDescription>
          </DialogHeader>
          {createdSecret ? (
            <div className="space-y-3">
              <div className="flex gap-2">
                <Input readOnly value={createdSecret} className="font-mono text-sm" />
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  onClick={() => {
                    void navigator.clipboard.writeText(createdSecret)
                    setCopied(true)
                  }}
                >
                  {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                </Button>
              </div>
              <Button
                type="button"
                variant="default"
                onClick={() => {
                  setCreatedSecret(null)
                  setCopied(false)
                  if (createdClientId) {
                    navigate(`/clients/${createdClientId}/settings`)
                  }
                }}
              >
                Go to client settings
              </Button>
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  )
}
