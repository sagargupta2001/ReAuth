import type { IdentityProvider, IdentityProviderPreset, IdentityProviderProtocol } from '@/entities/identity-provider/model/types'

interface IdentityProviderRealmDefaults {
  allow_jit_provisioning?: boolean
  allow_email_auto_link?: boolean
  require_verified_email?: boolean
  enabled?: boolean
}

export interface IdentityProviderFormValues {
  preset: string
  alias: string
  display_name: string
  protocol: IdentityProviderProtocol
  client_id: string
  client_secret: string
  issuer: string
  authorization_endpoint: string
  token_endpoint: string
  userinfo_endpoint: string
  jwks_uri: string
  scopes_input: string
  claim_mapping_input: string
  pkce_required: boolean
  allow_login: boolean
  allow_link: boolean
  allow_jit_provisioning: boolean
  allow_email_auto_link: boolean
  require_verified_email: boolean
  icon_ref: string
  button_color: string
  sort_order: number
  enabled: boolean
}

export interface IdentityProviderPayload {
  preset?: string
  alias: string
  display_name: string
  protocol: IdentityProviderProtocol
  client_id: string
  client_secret?: string
  issuer?: string
  authorization_endpoint?: string
  token_endpoint?: string
  userinfo_endpoint?: string
  jwks_uri?: string
  scopes: string[]
  claim_mapping: Record<string, unknown>
  pkce_required: boolean
  allow_login: boolean
  allow_link: boolean
  allow_jit_provisioning: boolean
  allow_email_auto_link: boolean
  require_verified_email: boolean
  icon_ref?: string
  button_color?: string
  sort_order: number
  enabled: boolean
}

export function parseScopesInput(input: string): string[] {
  return input
    .split(/[\n,]/)
    .map((value) => value.trim())
    .filter(Boolean)
}

export function serializeScopes(scopes: string[]): string {
  return scopes.join('\n')
}

export function parseClaimMappingInput(input: string): Record<string, unknown> {
  const trimmed = input.trim()
  if (!trimmed) return {}
  const parsed = JSON.parse(trimmed) as unknown
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('Claim mapping must be a JSON object.')
  }
  return parsed as Record<string, unknown>
}

export function serializeClaimMapping(mapping: Record<string, unknown>): string {
  return JSON.stringify(mapping, null, 2)
}

export function buildCreateDefaults(
  defaults: IdentityProviderRealmDefaults = {},
): IdentityProviderFormValues {
  return {
    preset: '',
    alias: '',
    display_name: '',
    protocol: 'oidc',
    client_id: '',
    client_secret: '',
    issuer: '',
    authorization_endpoint: '',
    token_endpoint: '',
    userinfo_endpoint: '',
    jwks_uri: '',
    scopes_input: 'openid\nemail\nprofile',
    claim_mapping_input: serializeClaimMapping({
      username: 'preferred_username',
      email: 'email',
      subject: 'sub',
    }),
    pkce_required: true,
    allow_login: true,
    allow_link: true,
    allow_jit_provisioning: defaults.allow_jit_provisioning ?? false,
    allow_email_auto_link: defaults.allow_email_auto_link ?? false,
    require_verified_email: defaults.require_verified_email ?? true,
    icon_ref: '',
    button_color: '',
    sort_order: 0,
    enabled: defaults.enabled ?? false,
  }
}

export function buildEditDefaults(provider: IdentityProvider): IdentityProviderFormValues {
  return {
    preset: provider.preset_key ?? '',
    alias: provider.alias,
    display_name: provider.display_name,
    protocol: provider.protocol,
    client_id: provider.client_id,
    client_secret: '',
    issuer: provider.issuer ?? '',
    authorization_endpoint: provider.authorization_endpoint ?? '',
    token_endpoint: provider.token_endpoint ?? '',
    userinfo_endpoint: provider.userinfo_endpoint ?? '',
    jwks_uri: provider.jwks_uri ?? '',
    scopes_input: serializeScopes(provider.scopes),
    claim_mapping_input: serializeClaimMapping(provider.claim_mapping),
    pkce_required: provider.pkce_required,
    allow_login: provider.allow_login,
    allow_link: provider.allow_link,
    allow_jit_provisioning: provider.allow_jit_provisioning,
    allow_email_auto_link: provider.allow_email_auto_link,
    require_verified_email: provider.require_verified_email,
    icon_ref: provider.icon_ref ?? '',
    button_color: provider.button_color ?? '',
    sort_order: provider.sort_order,
    enabled: provider.enabled,
  }
}

export function applyPresetToValues(
  values: IdentityProviderFormValues,
  preset: IdentityProviderPreset,
): IdentityProviderFormValues {
  return {
    ...values,
    preset: preset.key,
    display_name: preset.display_name,
    protocol: preset.protocol,
    issuer: preset.issuer ?? '',
    authorization_endpoint: preset.authorization_endpoint ?? '',
    token_endpoint: preset.token_endpoint ?? '',
    userinfo_endpoint: preset.userinfo_endpoint ?? '',
    jwks_uri: preset.jwks_uri ?? '',
    scopes_input: serializeScopes(preset.scopes),
    claim_mapping_input: serializeClaimMapping(preset.claim_mapping),
    icon_ref: preset.icon_ref ?? '',
  }
}

export function buildIdentityProviderPayload(
  values: IdentityProviderFormValues,
): IdentityProviderPayload {
  const scopes = parseScopesInput(values.scopes_input)
  if (scopes.length === 0) {
    throw new Error('At least one scope is required.')
  }

  return {
    preset: values.preset || undefined,
    alias: values.alias.trim(),
    display_name: values.display_name.trim(),
    protocol: values.protocol,
    client_id: values.client_id.trim(),
    client_secret: values.client_secret.trim() || undefined,
    issuer: values.issuer.trim() || undefined,
    authorization_endpoint: values.authorization_endpoint.trim() || undefined,
    token_endpoint: values.token_endpoint.trim() || undefined,
    userinfo_endpoint: values.userinfo_endpoint.trim() || undefined,
    jwks_uri: values.jwks_uri.trim() || undefined,
    scopes,
    claim_mapping: parseClaimMappingInput(values.claim_mapping_input),
    pkce_required: values.pkce_required,
    allow_login: values.allow_login,
    allow_link: values.allow_link,
    allow_jit_provisioning: values.allow_jit_provisioning,
    allow_email_auto_link: values.allow_email_auto_link,
    require_verified_email: values.require_verified_email,
    icon_ref: values.icon_ref.trim() || undefined,
    button_color: values.button_color.trim() || undefined,
    sort_order: values.sort_order,
    enabled: values.enabled,
  }
}
