import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, PlugZap, RefreshCcw } from 'lucide-react'
import { type Resolver, useForm } from 'react-hook-form'
import { toast } from 'sonner'

import type {
  IdentityProvider,
  IdentityProviderConnectionCheck,
} from '@/entities/identity-provider/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCurrentRealm } from '@/features/realm/api/useRealm'
import { useCreateIdentityProvider } from '@/features/identity-provider/api/useCreateIdentityProvider'
import { useIdentityProviderPresets } from '@/features/identity-provider/api/useIdentityProviderPresets'
import { useRefreshIdentityProviderMetadata } from '@/features/identity-provider/api/useRefreshIdentityProviderMetadata'
import { useTestIdentityProviderConnection } from '@/features/identity-provider/api/useTestIdentityProviderConnection'
import { useUpdateIdentityProvider } from '@/features/identity-provider/api/useUpdateIdentityProvider'
import {
  applyPresetToValues,
  buildCreateDefaults,
  buildEditDefaults,
  buildIdentityProviderPayload,
  type IdentityProviderFormValues,
} from '@/features/identity-provider/lib/form'
import {
  identityProviderFormSchema,
  type IdentityProviderFormSchema,
} from '@/features/identity-provider/schema/identity-provider.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { Switch } from '@/shared/ui/switch'

interface Props {
  provider?: IdentityProvider
}

function formatTimestamp(value?: string | null) {
  return value ? new Date(value).toLocaleString() : 'Not cached yet'
}

function CheckBadge({ check }: { check: IdentityProviderConnectionCheck }) {
  if (!check.attempted) {
    return <Badge variant="muted">Skipped</Badge>
  }
  return <Badge variant={check.ok ? 'success' : 'destructive'}>{check.ok ? 'OK' : 'Failed'}</Badge>
}

function SwitchField({
  control,
  name,
  label,
  description,
}: {
  control: ReturnType<typeof useForm<IdentityProviderFormSchema>>['control']
  name:
    | 'enabled'
    | 'pkce_required'
    | 'allow_login'
    | 'allow_link'
    | 'allow_jit_provisioning'
    | 'allow_email_auto_link'
    | 'require_verified_email'
  label: string
  description: string
}) {
  return (
    <FormField
      control={control}
      name={name}
      render={({ field }) => (
        <FormItem className="flex items-center justify-between rounded-lg border p-4">
          <div className="space-y-1">
            <FormLabel>{label}</FormLabel>
            <FormDescription>{description}</FormDescription>
          </div>
          <FormControl>
            <Switch checked={Boolean(field.value)} onCheckedChange={field.onChange} />
          </FormControl>
        </FormItem>
      )}
    />
  )
}

export function IdentityProviderForm({ provider }: Props) {
  const navigate = useRealmNavigate()
  const { data: realm } = useCurrentRealm()
  const createMutation = useCreateIdentityProvider()
  const updateMutation = useUpdateIdentityProvider(provider?.id || '')
  const refreshMutation = useRefreshIdentityProviderMetadata(provider?.id || '')
  const testConnectionMutation = useTestIdentityProviderConnection(provider?.id || '')
  const { data: presets = [], isLoading: presetsLoading } = useIdentityProviderPresets()

  const form = useForm<IdentityProviderFormSchema>({
    resolver: zodResolver(identityProviderFormSchema) as Resolver<IdentityProviderFormSchema>,
    defaultValues: buildCreateDefaults(),
  })

  useEffect(() => {
    if (provider) {
      form.reset(buildEditDefaults(provider))
      return
    }
    if (!realm) {
      form.reset(buildCreateDefaults())
      return
    }
    form.reset(
      buildCreateDefaults({
        allow_jit_provisioning: realm.idp_default_jit_policy === 'allow',
        allow_email_auto_link: realm.idp_default_email_link_policy === 'allow_verified',
        require_verified_email: realm.idp_default_email_link_policy !== 'deny',
        enabled: realm.idp_broker_enabled,
      }),
    )
  }, [provider, realm, form])

  const onSubmit = async (values: IdentityProviderFormValues) => {
    let payload
    try {
      payload = buildIdentityProviderPayload(values)
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid provider configuration.'
      if (message.includes('Claim mapping')) {
        form.setError('claim_mapping_input', { message })
      } else if (message.includes('scope')) {
        form.setError('scopes_input', { message })
      } else {
        toast.error(message)
      }
      return
    }

    if (provider) {
      updateMutation.mutate(payload, {
        onSuccess: (savedProvider) => {
          form.reset(buildEditDefaults(savedProvider))
        },
      })
      return
    }

    createMutation.mutate(payload, {
      onSuccess: (savedProvider) => {
        form.reset(buildEditDefaults(savedProvider))
        navigate(`/identity-providers/${savedProvider.id}`)
      },
    })
  }

  useFormPersistence(form, onSubmit, createMutation.isPending || updateMutation.isPending)

  const applyPreset = (presetKey: string) => {
    if (presetKey === '__none__') {
      form.setValue('preset', '')
      return
    }
    const preset = presets.find((entry) => entry.key === presetKey)
    if (!preset) return
    form.reset(applyPresetToValues(form.getValues(), preset))
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
        <Card>
          <CardHeader className="flex flex-row items-start justify-between gap-4">
            <div>
              <CardTitle>{provider ? 'Edit Identity Provider' : 'Create Identity Provider'}</CardTitle>
              <CardDescription>
                Configure an inbound OAuth or OIDC provider for this realm.
              </CardDescription>
            </div>
            {provider ? (
              <div className="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => testConnectionMutation.mutate()}
                  disabled={testConnectionMutation.isPending}
                  className="gap-2"
                >
                  {testConnectionMutation.isPending ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <PlugZap className="h-4 w-4" />
                  )}
                  Test Connection
                </Button>
                {provider.protocol === 'oidc' ? (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => refreshMutation.mutate()}
                    disabled={refreshMutation.isPending}
                    className="gap-2"
                  >
                    {refreshMutation.isPending ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <RefreshCcw className="h-4 w-4" />
                    )}
                    Refresh Metadata
                  </Button>
                ) : null}
              </div>
            ) : null}
          </CardHeader>
          <CardContent className="grid gap-6 md:grid-cols-2">
            <FormField
              control={form.control}
              name="preset"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Preset</FormLabel>
                  <Select
                    value={field.value || '__none__'}
                    onValueChange={(value) => {
                      applyPreset(value)
                      field.onChange(value === '__none__' ? '' : value)
                    }}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder={presetsLoading ? 'Loading presets...' : 'Choose a preset'} />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="__none__">No preset</SelectItem>
                      {presets.map((preset) => (
                        <SelectItem key={preset.key} value={preset.key}>
                          {preset.display_name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    Presets seed protocol, scopes, endpoints, claim mapping, and icon defaults.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="protocol"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Protocol</FormLabel>
                  <Select value={field.value} onValueChange={field.onChange}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="oidc">OIDC</SelectItem>
                      <SelectItem value="oauth2">OAuth2</SelectItem>
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormInput control={form.control} name="alias" label="Alias" description="URL-safe path segment used in callback routes." />
            <FormInput control={form.control} name="display_name" label="Display Name" description="Human-readable label shown to admins and end users." />
            <FormInput control={form.control} name="client_id" label="Client ID" />
            <FormInput
              control={form.control}
              name="client_secret"
              type="password"
              label="Client Secret"
              description={
                provider?.client_secret_set
                  ? `Secret is set (${provider.client_secret_mask || 'hidden'}). Leave blank to keep the current value.`
                  : 'Optional for public providers; stored encrypted at rest.'
              }
            />
            <FormInput control={form.control} name="icon_ref" label="Icon Ref" description="Preset icon slug or custom asset reference for login buttons." />
            <FormInput control={form.control} name="button_color" label="Button Color" description="Optional hex or CSS color token for the login button." />
            <FormInput control={form.control} name="sort_order" type="number" label="Sort Order" description="Lower values render first on the login page." />
          </CardContent>
        </Card>

        {provider ? (
          <Card>
            <CardHeader>
              <CardTitle>Runtime Status</CardTitle>
              <CardDescription>
                Cache freshness and the latest admin connection test result for this provider.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-4 md:grid-cols-3">
                <div>
                  <div className="text-muted-foreground text-xs uppercase tracking-wide">Client Secret</div>
                  <div className="text-sm">
                    {provider.client_secret_set
                      ? `Configured ${provider.client_secret_mask ? `(${provider.client_secret_mask})` : ''}`
                      : 'Not configured'}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground text-xs uppercase tracking-wide">Discovery Cache</div>
                  <div className="text-sm">{formatTimestamp(provider.metadata_cached_at)}</div>
                </div>
                <div>
                  <div className="text-muted-foreground text-xs uppercase tracking-wide">JWKS Cache</div>
                  <div className="text-sm">{formatTimestamp(provider.jwks_cached_at)}</div>
                </div>
              </div>

              {testConnectionMutation.data ? (
                <div className="rounded-lg border p-4">
                  <div className="mb-4 flex flex-wrap items-center gap-2">
                    <Badge variant={testConnectionMutation.data.ok ? 'success' : 'destructive'}>
                      {testConnectionMutation.data.ok ? 'Connection Healthy' : 'Connection Issues'}
                    </Badge>
                    <span className="text-muted-foreground text-sm">
                      Tested {new Date(testConnectionMutation.data.tested_at).toLocaleString()}
                    </span>
                  </div>
                  <div className="grid gap-4 md:grid-cols-2">
                    {[
                      { label: 'Discovery', check: testConnectionMutation.data.discovery },
                      { label: 'Token Endpoint', check: testConnectionMutation.data.token_endpoint },
                      { label: 'Userinfo Endpoint', check: testConnectionMutation.data.userinfo_endpoint },
                      { label: 'JWKS', check: testConnectionMutation.data.jwks },
                    ].map(({ label, check }) => (
                      <div key={label} className="rounded-md border p-3">
                        <div className="mb-2 flex items-center justify-between gap-2">
                          <div className="text-sm font-medium">{label}</div>
                          <CheckBadge check={check} />
                        </div>
                        <div className="text-muted-foreground text-xs">
                          {check.detail}
                          {check.status_code ? ` · HTTP ${check.status_code}` : ''}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </CardContent>
          </Card>
        ) : null}

        <Card>
          <CardHeader>
            <CardTitle>Endpoints and Claims</CardTitle>
            <CardDescription>
              Override preset metadata as needed. OIDC providers can also refresh metadata from discovery.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-6 md:grid-cols-2">
            <FormInput control={form.control} name="issuer" label="Issuer URL" description="Required for OIDC discovery-based providers." />
            <FormInput control={form.control} name="jwks_uri" label="JWKS URI" description="Used to validate inbound OIDC id_tokens." />
            <div className="md:col-span-2">
              <FormInput control={form.control} name="authorization_endpoint" label="Authorization Endpoint" />
            </div>
            <div className="md:col-span-2">
              <FormInput control={form.control} name="token_endpoint" label="Token Endpoint" />
            </div>
            <div className="md:col-span-2">
              <FormInput control={form.control} name="userinfo_endpoint" label="Userinfo Endpoint" />
            </div>
            <div className="md:col-span-2">
              <FormTextarea
                control={form.control}
                name="scopes_input"
                label="Scopes"
                description="Enter one scope per line or separate them with commas."
                rows={5}
              />
            </div>
            <div className="md:col-span-2">
              <FormTextarea
                control={form.control}
                name="claim_mapping_input"
                label="Claim Mapping JSON"
                description='JSON object mapping ReAuth fields to upstream claim paths, for example {"email":"email","username":"preferred_username"}.'
                rows={12}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Access and Linking</CardTitle>
            <CardDescription>
              Control whether this provider can be used for login, linking, and just-in-time account creation.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2">
            <SwitchField
              control={form.control}
              name="enabled"
              label="Enabled"
              description="Controls whether the provider is available to browser flows in this realm."
            />
            <SwitchField
              control={form.control}
              name="pkce_required"
              label="Require PKCE"
              description="Recommended for all public browser flows and most confidential clients too."
            />
            <SwitchField
              control={form.control}
              name="allow_login"
              label="Allow Login"
              description="Shows this provider as a valid sign-in option when referenced by the flow."
            />
            <SwitchField
              control={form.control}
              name="allow_link"
              label="Allow Linking"
              description="Lets this provider be attached to an existing local ReAuth account."
            />
            <SwitchField
              control={form.control}
              name="allow_jit_provisioning"
              label="Allow JIT Provisioning"
              description="Creates a local user when no existing account match is found."
            />
            <SwitchField
              control={form.control}
              name="allow_email_auto_link"
              label="Allow Email Auto-Link"
              description="Auto-links to a local user when the upstream email matches."
            />
            <div className="md:col-span-2">
              <SwitchField
                control={form.control}
                name="require_verified_email"
                label="Require Verified Email"
                description="Require the upstream provider to assert a verified email before email-based matching is trusted."
              />
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  )
}
