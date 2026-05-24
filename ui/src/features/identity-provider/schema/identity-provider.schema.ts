import { z } from 'zod'

export const identityProviderFormSchema = z.object({
  preset: z.string(),
  alias: z
    .string()
    .min(1, 'Alias is required.')
    .max(64, 'Alias must be 64 characters or fewer.')
    .regex(/^[a-z0-9_-]+$/, 'Only lowercase letters, numbers, hyphens, and underscores allowed.'),
  display_name: z.string().min(1, 'Display name is required.'),
  protocol: z.enum(['oidc', 'oauth2']),
  client_id: z.string().min(1, 'Client ID is required.'),
  client_secret: z.string(),
  issuer: z.string(),
  authorization_endpoint: z.string(),
  token_endpoint: z.string(),
  userinfo_endpoint: z.string(),
  jwks_uri: z.string(),
  scopes_input: z.string().min(1, 'At least one scope is required.'),
  claim_mapping_input: z.string().min(2, 'Claim mapping JSON is required.'),
  pkce_required: z.boolean(),
  allow_login: z.boolean(),
  allow_link: z.boolean(),
  allow_jit_provisioning: z.boolean(),
  allow_email_auto_link: z.boolean(),
  require_verified_email: z.boolean(),
  icon_ref: z.string(),
  button_color: z.string(),
  sort_order: z.coerce.number().int('Sort order must be a whole number'),
  enabled: z.boolean(),
})

export type IdentityProviderFormSchema = z.infer<typeof identityProviderFormSchema>
