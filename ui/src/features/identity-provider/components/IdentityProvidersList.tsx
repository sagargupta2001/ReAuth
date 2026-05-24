import { Loader2, Pencil, RefreshCcw } from 'lucide-react'

import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useRefreshIdentityProviderMetadata } from '@/features/identity-provider/api/useRefreshIdentityProviderMetadata'
import { useIdentityProviders } from '@/features/identity-provider/api/useIdentityProviders'

function RefreshMetadataButton({
  providerId,
  disabled,
}: {
  providerId: string
  disabled: boolean
}) {
  const refresh = useRefreshIdentityProviderMetadata(providerId)

  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      disabled={disabled || refresh.isPending}
      onClick={() => refresh.mutate()}
      className="gap-2"
    >
      <RefreshCcw className="h-4 w-4" />
      Refresh Metadata
    </Button>
  )
}

export function IdentityProvidersList() {
  const navigate = useRealmNavigate()
  const { data: providers, isLoading, isError } = useIdentityProviders()

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-64 items-center justify-center gap-3">
        <Loader2 className="h-5 w-5 animate-spin" />
        <span>Loading identity providers...</span>
      </div>
    )
  }

  if (isError) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Failed to load identity providers</CardTitle>
          <CardDescription>Refresh the page or try again in a moment.</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  if (!providers?.length) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>No identity providers yet</CardTitle>
          <CardDescription>
            Add Google, GitHub, Microsoft, or a custom OAuth/OIDC provider for this realm.
          </CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <div className="grid gap-4">
      {providers.map((provider) => (
        <Card key={provider.id}>
          <CardHeader className="flex flex-row items-start justify-between gap-4">
            <div className="space-y-2">
              <div className="flex flex-wrap items-center gap-2">
                <CardTitle>{provider.display_name}</CardTitle>
                <Badge variant={provider.enabled ? 'success' : 'muted'}>
                  {provider.enabled ? 'Enabled' : 'Disabled'}
                </Badge>
                <Badge variant="outline">{provider.protocol.toUpperCase()}</Badge>
                {provider.preset_key ? <Badge variant="cool">{provider.preset_key}</Badge> : null}
              </div>
              <CardDescription>
                Alias: <span className="font-mono">{provider.alias}</span>
              </CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <RefreshMetadataButton
                providerId={provider.id}
                disabled={provider.protocol !== 'oidc' || !provider.issuer}
              />
              <Button variant="outline" size="sm" onClick={() => navigate(`/identity-providers/${provider.id}`)}>
                <Pencil className="mr-2 h-4 w-4" />
                Edit
              </Button>
            </div>
          </CardHeader>
          <CardContent className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <div>
              <div className="text-muted-foreground text-xs uppercase tracking-wide">Client ID</div>
              <div className="truncate text-sm">{provider.client_id}</div>
            </div>
            <div>
              <div className="text-muted-foreground text-xs uppercase tracking-wide">Metadata</div>
              <div className="text-sm">
                {provider.metadata_cached_at
                  ? new Date(provider.metadata_cached_at).toLocaleString()
                  : 'Not refreshed yet'}
              </div>
            </div>
            <div>
              <div className="text-muted-foreground text-xs uppercase tracking-wide">Scopes</div>
              <div className="truncate text-sm">{provider.scopes.join(', ')}</div>
            </div>
            <div>
              <div className="text-muted-foreground text-xs uppercase tracking-wide">Button</div>
              <div className="text-sm">
                {provider.icon_ref || 'No icon'} {provider.button_color ? `· ${provider.button_color}` : ''}
              </div>
            </div>
            <div className="md:col-span-2 xl:col-span-4">
              <div className="flex flex-wrap gap-2">
                <Badge variant={provider.allow_login ? 'info' : 'muted'}>
                  {provider.allow_login ? 'Login allowed' : 'Login blocked'}
                </Badge>
                <Badge variant={provider.allow_link ? 'info' : 'muted'}>
                  {provider.allow_link ? 'Link allowed' : 'Link blocked'}
                </Badge>
                <Badge variant={provider.allow_jit_provisioning ? 'info' : 'muted'}>
                  {provider.allow_jit_provisioning ? 'JIT enabled' : 'JIT disabled'}
                </Badge>
                <Badge variant={provider.allow_email_auto_link ? 'info' : 'muted'}>
                  {provider.allow_email_auto_link ? 'Email auto-link' : 'Manual email link'}
                </Badge>
                <Badge variant={provider.pkce_required ? 'cool' : 'muted'}>
                  {provider.pkce_required ? 'PKCE required' : 'PKCE optional'}
                </Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
