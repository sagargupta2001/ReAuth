import type { IdentityProvider } from '@/entities/identity-provider/model/types'

/**
 * Minimal provider shape the `ProviderButtons` component needs to render.
 *
 * The runtime gets this from the auth context's `enabled_providers`; the builder
 * canvas gets it from the realm's configured providers so the preview matches
 * what an end user will actually see.
 */
export interface ProviderPreview {
  alias: string
  display_name: string
  button_color?: string | null
}

/** Providers that will appear on a login page, in their configured order. */
export function toProviderPreviews(providers: IdentityProvider[]): ProviderPreview[] {
  return providers
    .filter((provider) => provider.enabled && provider.allow_login)
    .slice()
    .sort((a, b) => a.sort_order - b.sort_order)
    .map(({ alias, display_name, button_color }) => ({ alias, display_name, button_color }))
}
