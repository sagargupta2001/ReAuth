import { useEffect, useMemo, useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, Plus, Trash2 } from 'lucide-react'
import { useFieldArray, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/form'
import { Input } from '@/components/input'
import type { OidcClient } from '@/entities/oidc/model/types.ts'
import type { ThemeSnapshot } from '@/entities/theme/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useUpdateClient } from '@/features/client/api/useUpdateClient.ts'
import { useRotateClientSecret } from '@/features/client/api/useRotateClientSecret'
import { useThemePages } from '@/features/theme/api/useThemePages'
import { useThemes } from '@/features/theme/api/useThemes'
import { useThemeVersions } from '@/features/theme/api/useThemeVersions'
import { useUpsertThemeBinding } from '@/features/theme/api/useUpsertThemeBinding'
import { useDeleteThemeBinding } from '@/features/theme/api/useDeleteThemeBinding'
import { useClientThemeBinding } from '@/features/theme/api/useClientThemeBinding'
import {
  type CreateClientSchema,
  createClientSchema,
} from '@/features/client/schema/create.schema.ts'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { Label } from '@/shared/ui/label.tsx'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'
import { apiClient } from '@/shared/api/client'

import { ClientSecretInput } from './ClientSecretInput'

interface ClientSettingsTabProps {
  client: OidcClient
}

export function ClientSettingsTab({ client }: ClientSettingsTabProps) {
  const { t } = useTranslation('client')
  const mutation = useUpdateClient(client.id)
  const realm = useActiveRealm()
  const { data: themes = [] } = useThemes()
  const { data: binding } = useClientThemeBinding(client.client_id)
  const [selectedThemeId, setSelectedThemeId] = useState('')
  const [selectedVersionId, setSelectedVersionId] = useState('')
  const { data: versions = [] } = useThemeVersions(selectedThemeId)
  const { data: pages = [] } = useThemePages()
  const upsertBinding = useUpsertThemeBinding(selectedThemeId)
  const deleteBinding = useDeleteThemeBinding(binding?.theme_id || selectedThemeId)
  const [previewOpen, setPreviewOpen] = useState(false)
  const [previewPageKey, setPreviewPageKey] = useState('login')
  const [clientSecret, setClientSecret] = useState<string | null>(
    client.client_secret ?? null,
  )
  const rotateSecret = useRotateClientSecret(client.id)

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

  useEffect(() => {
    setClientSecret(client.client_secret ?? null)
  }, [client.client_secret])

  useEffect(() => {
    if (selectedThemeId) return
    const nextThemeId = binding?.theme_id || themes[0]?.id || ''
    if (nextThemeId) {
      setSelectedThemeId(nextThemeId)
    }
  }, [binding, themes, selectedThemeId])

  useEffect(() => {
    if (selectedVersionId) return
    const nextVersionId = binding?.active_version_id || versions[0]?.id || ''
    if (nextVersionId) {
      setSelectedVersionId(nextVersionId)
    }
  }, [binding, versions, selectedVersionId])

  useEffect(() => {
    if (pages.length === 0) return
    if (!pages.some((page) => page.key === previewPageKey)) {
      setPreviewPageKey(pages[0].key)
    }
  }, [pages, previewPageKey])

  const bindingTheme = useMemo(
    () => themes.find((theme) => theme.id === binding?.theme_id) || null,
    [themes, binding],
  )

  const previewQuery = useMemo(() => {
    if (!realm) return null
    const search = new URLSearchParams()
    search.set('client_id', client.client_id)
    search.set('page_key', previewPageKey || 'login')
    return `/api/realms/${realm}/theme/resolve?${search.toString()}`
  }, [realm, client.client_id, previewPageKey])

  const [previewState, setPreviewState] = useState<{
    data: ThemeSnapshot | null
    loading: boolean
  }>({ data: null, loading: false })

  useEffect(() => {
    let cancelled = false
    if (!previewOpen || !previewQuery) {
      setPreviewState({ data: null, loading: false })
      return
    }
    setPreviewState({ data: null, loading: true })
    apiClient
      .get(previewQuery)
      .then((data) => {
        if (!cancelled) {
          setPreviewState({ data: data as ThemeSnapshot, loading: false })
        }
      })
      .catch(() => {
        if (!cancelled) {
          setPreviewState({ data: null, loading: false })
        }
      })
    return () => {
      cancelled = true
    }
  }, [previewOpen, previewQuery])

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

  const handleSaveOverride = () => {
    if (!selectedThemeId || !selectedVersionId) return
    upsertBinding.mutate({
      clientId: client.client_id,
      versionId: selectedVersionId,
    })
  }

  const handleRemoveOverride = () => {
    if (!binding) return
    deleteBinding.mutate(client.client_id)
  }

  const handleRotateSecret = async () => {
    try {
      const result = await rotateSecret.mutateAsync()
      setClientSecret(result.client_secret ?? null)
    } catch {
      // handled in hook
    }
  }

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
              <ClientSecretInput
                secret={clientSecret}
                confidential={client.confidential}
                onRotate={handleRotateSecret}
                isRotating={rotateSecret.isPending}
              />
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

          <Card>
            <CardHeader>
              <CardTitle>Theme Override</CardTitle>
              <CardDescription>
                Assign a specific theme version for this client. Without an override, the realm
                default theme will be used.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-3 md:grid-cols-[1fr_160px_auto]">
                <Select
                  value={selectedThemeId}
                  onValueChange={(value) => {
                    setSelectedThemeId(value)
                    setSelectedVersionId('')
                  }}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Theme" />
                  </SelectTrigger>
                  <SelectContent>
                    {themes.map((theme) => (
                      <SelectItem key={theme.id} value={theme.id}>
                        {theme.name || theme.id}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <Select value={selectedVersionId} onValueChange={setSelectedVersionId}>
                  <SelectTrigger>
                    <SelectValue placeholder="Version" />
                  </SelectTrigger>
                  <SelectContent>
                    {versions.map((version) => (
                      <SelectItem key={version.id} value={version.id}>
                        v{version.version_number}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>

                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    onClick={handleSaveOverride}
                    disabled={!selectedThemeId || !selectedVersionId || upsertBinding.isPending}
                  >
                    {upsertBinding.isPending ? (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    ) : null}
                    Save override
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleRemoveOverride}
                    disabled={!binding || deleteBinding.isPending}
                  >
                    Remove
                  </Button>
                </div>
              </div>

              <div className="rounded-md border p-3 text-xs">
                {binding ? (
                  <div className="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                    <div className="space-y-1">
                      <div className="font-semibold">
                        {bindingTheme?.name || binding.theme_id}
                      </div>
                      <div className="text-muted-foreground">
                        Version v{binding.active_version_number ?? 'unknown'}
                      </div>
                    </div>
                    <Button type="button" size="sm" variant="outline" onClick={() => setPreviewOpen(true)}>
                      Preview
                    </Button>
                  </div>
                ) : (
                  <div className="text-muted-foreground">
                    No override configured. This client uses the realm default theme.
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>

      <Dialog open={previewOpen} onOpenChange={setPreviewOpen}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>Client Theme Preview</DialogTitle>
          </DialogHeader>
          <div className="mb-3 flex items-center gap-2">
            <Select value={previewPageKey} onValueChange={setPreviewPageKey}>
              <SelectTrigger className="w-[220px]">
                <SelectValue placeholder="Page" />
              </SelectTrigger>
              <SelectContent>
                {pages.map((page) => (
                  <SelectItem key={page.key} value={page.key}>
                    {page.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="h-[540px] overflow-hidden rounded-lg border">
            {previewState.loading ? (
              <div className="text-muted-foreground flex h-full items-center justify-center">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" /> Loading preview...
              </div>
            ) : previewState.data ? (
              <FluidCanvas
                tokens={previewState.data.tokens}
                layout={previewState.data.layout}
                blocks={previewState.data.nodes}
                assets={previewState.data.assets}
                selectedNodeId={null}
                isInspecting={false}
                showChrome={false}
                onSelectNode={() => {}}
              />
            ) : (
              <div className="text-muted-foreground flex h-full items-center justify-center">
                Unable to load preview.
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
