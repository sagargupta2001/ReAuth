import { describe, expect, it } from 'vitest'

import { toProviderPreviews } from './providerPreview'
import type { IdentityProvider } from '@/entities/identity-provider/model/types'

function provider(overrides: Partial<IdentityProvider>): IdentityProvider {
  return {
    id: 'idp-1',
    realm_id: 'realm-1',
    alias: 'google',
    display_name: 'Google',
    protocol: 'oidc',
    enabled: true,
    client_id: 'client',
    scopes: [],
    claim_mapping: {},
    pkce_required: true,
    allow_login: true,
    allow_link: true,
    allow_jit_provisioning: true,
    allow_email_auto_link: false,
    require_verified_email: false,
    sort_order: 0,
    client_secret_set: true,
    ...overrides,
  }
}

describe('toProviderPreviews', () => {
  it('keeps only providers that can be used to sign in', () => {
    const previews = toProviderPreviews([
      provider({ alias: 'google' }),
      provider({ alias: 'disabled', enabled: false }),
      provider({ alias: 'link-only', allow_login: false }),
    ])

    expect(previews.map((p) => p.alias)).toEqual(['google'])
  })

  it('sorts by configured order', () => {
    const previews = toProviderPreviews([
      provider({ alias: 'okta', sort_order: 2 }),
      provider({ alias: 'google', sort_order: 1 }),
    ])

    expect(previews.map((p) => p.alias)).toEqual(['google', 'okta'])
  })

  it('does not mutate the input array', () => {
    const providers = [
      provider({ alias: 'okta', sort_order: 2 }),
      provider({ alias: 'google', sort_order: 1 }),
    ]
    toProviderPreviews(providers)
    expect(providers.map((p) => p.alias)).toEqual(['okta', 'google'])
  })

  it('carries the button color through', () => {
    const [preview] = toProviderPreviews([provider({ button_color: '#4285F4' })])
    expect(preview).toEqual({ alias: 'google', display_name: 'Google', button_color: '#4285F4' })
  })
})
