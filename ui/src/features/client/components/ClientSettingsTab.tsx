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
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useDeleteClient } from '@/features/client/api/useDeleteClient'
import { useClientDeleteSummary } from '@/features/client/api/useClientDeleteSummary'
import { useUpdateClient } from '@/features/client/api/useUpdateClient.ts'
import { useRotateClientSecret } from '@/features/client/api/useRotateClientSecret'
import { parseJsonArray } from '@/features/client/lib/clientFields'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import { useThemeSnapshot } from '@/features/theme/api/useThemeSnapshot'
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/dialog'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'

import { ClientSecretInput } from './ClientSecretInput'

interface ClientSettingsTabProps {
  client: OidcClient
}

export function ClientSettingsTab({ client }: ClientSettingsTabProps) {
  const { t } = useTranslation('client')
  const mutation = useUpdateClient(client.id)
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()
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

  const [deleteOpen, setDeleteOpen] = useState(false)
  const deleteClient = useDeleteClient(client.id)
  const { data: deleteSummary, isLoading: deleteSummaryLoading } = useClientDeleteSummary(
    client.id,
    deleteOpen,
  )

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema()),
    defaultValues: {
      client_id: client.client_id,
      redirect_uris: [{ value: '' }],
      web_origins: [{ value: '' }],
      scopes: [{ value: '' }],
    },
  })

  const redirectUriFields = useFieldArray({ control: form.control, name: 'redirect_uris' })
  const webOriginFields = useFieldArray({ control: form.control, name: 'web_origins' })
  const scopeFields = useFieldArray({ control: form.control, name: 'scopes' })

  useEffect(() => {
    const uris = parseJsonArray(client.redirect_uris)
    const origins = parseJsonArray(client.web_origins)
    const scopes = parseJsonArray(client.scopes)

    form.reset({
      client_id: client.client_id,
      redirect_uris: uris.length ? uris.map((u) => ({ value: u })) : [{ value: '' }],
      web_origins: origins.length ? origins.map((u) => ({ value: u })) : [{ value: '' }],
      scopes: scopes.length ? scopes.map((s) => ({ value: s })) : [{ value: '' }],
    })
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

  const previewEnabled = previewOpen && !!realm
  const { data: previewSnapshot, isLoading: isPreviewLoading } = useThemeSnapshot(
    realm,
    {
      clientId: client.client_id,
      pageKey: previewPageKey || 'login',
    },
    { enabled: previewEnabled },
  )

  const onSubmit = (values: CreateClientSchema) => {
    mutation.mutate(
      {
        client_id: values.client_id,
        redirect_uris: values.redirect_uris.map((u) => u.value),
        web_origins: values.web_origins?.map((u) => u.value) || [],
        scopes: values.scopes?.map((s) => s.value).filter(Boolean) || [],
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

  const handleConfirmDelete = () => {
    deleteClient.mutate(undefined, {
      onSuccess: () => {
        setDeleteOpen(false)
        navigate('/clients')
      },
    })
  }

  return (
    <div className="space-y-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          {/* Section 1: Identity */}
          <Card>
            <CardHeader>
              <CardTitle>Client Identity</CardTitle>
              <CardDescription>Core credentials for OIDC authentication.</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground space-y-6 rounded-2xl p-4">
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
              </div>
            </CardContent>
          </Card>

          {/* Section 2: Access & Security */}
          <Card>
            <CardHeader>
              <CardTitle>Access Settings</CardTitle>
              <CardDescription>Configure allowed URLs and CORS policies.</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground space-y-6 rounded-2xl p-4">
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
              </div>
            </CardContent>
          </Card>

          {/* Section 3: Scopes */}
          <Card>
            <CardHeader>
              <CardTitle>Scopes</CardTitle>
              <CardDescription>
                OAuth scopes this client is allowed to request (e.g. openid, profile, email).
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground space-y-4 rounded-2xl p-4">
                <div className="flex items-center justify-between">
                  <Label>Allowed Scopes</Label>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => scopeFields.append({ value: '' })}
                  >
                    <Plus className="mr-2 h-3.5 w-3.5" /> Add Scope
                  </Button>
                </div>
                <div className="space-y-2">
                  {scopeFields.fields.map((field, index) => (
                    <div key={field.id} className="flex gap-2">
                      <FormField
                        control={form.control}
                        name={`scopes.${index}.value`}
                        render={({ field }) => (
                          <FormItem className="flex-1 space-y-0">
                            <FormControl>
                              <Input placeholder="openid" {...field} />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => scopeFields.remove(index)}
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
            <CardContent>
              <div className="bg-primary-foreground space-y-4 rounded-2xl p-4">
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
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => setPreviewOpen(true)}
                      >
                        Preview
                      </Button>
                    </div>
                  ) : (
                    <div className="text-muted-foreground">
                      No override configured. This client uses the realm default theme.
                    </div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>

      <Card>
        <CardHeader>
          <CardTitle>Harbor</CardTitle>
          <CardDescription>
            Export this client&apos;s configuration or import a definition to overwrite it.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="bg-primary-foreground flex items-center justify-between gap-4 rounded-2xl p-4">
            <p className="text-muted-foreground text-sm">
              Download the flow definition or upload a bundle to apply changes.
            </p>
            <HarborResourceActions
              scope="client"
              id={client.client_id}
              resourceLabel={client.client_id}
              allowedConflictPolicies={['overwrite', 'skip']}
              invalidateKeys={[
                ['client', realm, client.id],
                ['clients', realm],
              ]}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Danger Zone</CardTitle>
          <CardDescription>
            Delete this client and everything scoped to it.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="border-destructive/30 bg-destructive/5 flex flex-wrap items-center justify-between gap-4 rounded-2xl border p-4">
            <div>
              <p className="text-sm font-medium">Delete client</p>
              <p className="text-muted-foreground text-sm">
                Permanently removes the client and cascades to its client-scoped roles and
                permissions.
              </p>
            </div>
            <Button type="button" variant="destructive" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="h-4 w-4" />
              Delete Client
            </Button>
          </div>
        </CardContent>
      </Card>

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent className="sm:max-w-[520px]">
          <DialogHeader className="px-6 pt-6">
            <DialogTitle>Delete client</DialogTitle>
            <DialogDescription>
              This permanently removes <span className="font-medium">{client.client_id}</span> and
              cascades to the client-scoped roles and permissions below.
            </DialogDescription>
          </DialogHeader>

          <div className="px-6 pb-2">
            {deleteSummaryLoading ? (
              <div className="text-muted-foreground text-sm">Loading impact...</div>
            ) : deleteSummary ? (
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Client roles</div>
                  <div className="font-medium">{deleteSummary.role_count}</div>
                </div>
                <div className="rounded-md border px-3 py-2">
                  <div className="text-muted-foreground text-xs">Client permissions</div>
                  <div className="font-medium">{deleteSummary.permission_count}</div>
                </div>
              </div>
            ) : (
              <div className="text-destructive text-sm">Unable to load delete impact.</div>
            )}
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleteSummaryLoading || deleteClient.isPending || !deleteSummary}
            >
              {deleteClient.isPending ? 'Deleting...' : 'Delete Client'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

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
            {isPreviewLoading ? (
              <div className="text-muted-foreground flex h-full items-center justify-center">
                <Loader2 className="mr-2 h-4 w-4 animate-spin" /> Loading preview...
              </div>
            ) : previewSnapshot ? (
              <FluidCanvas
                tokens={previewSnapshot.tokens}
                layout={previewSnapshot.layout}
                blocks={previewSnapshot.nodes}
                assets={previewSnapshot.assets}
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
