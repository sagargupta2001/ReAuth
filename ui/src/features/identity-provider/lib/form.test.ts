import { describe, expect, it } from 'vitest'

import {
  applyPresetToValues,
  buildCreateDefaults,
  buildIdentityProviderPayload,
  parseClaimMappingInput,
  parseScopesInput,
} from './form'

describe('identity provider form helpers', () => {
  it('parses scopes from newlines and commas', () => {
    expect(parseScopesInput('openid,\n email\nprofile , custom')).toEqual([
      'openid',
      'email',
      'profile',
      'custom',
    ])
  })

  it('requires claim mapping to be a json object', () => {
    expect(() => parseClaimMappingInput('["email"]')).toThrow(
      'Claim mapping must be a JSON object.',
    )
  })

  it('builds a provider payload from form values', () => {
    const defaults = buildCreateDefaults()
    const payload = buildIdentityProviderPayload({
      ...defaults,
      alias: 'google',
      display_name: 'Google',
      client_id: 'client-google',
      scopes_input: 'openid\nemail',
    })

    expect(payload.scopes).toEqual(['openid', 'email'])
    expect(payload.alias).toBe('google')
    expect(payload.claim_mapping).toMatchObject({
      username: 'preferred_username',
      email: 'email',
    })
  })

  it('applies a preset onto draft values', () => {
    const values = buildCreateDefaults()
    const updated = applyPresetToValues(values, {
      key: 'github',
      display_name: 'GitHub',
      protocol: 'oauth2',
      issuer: null,
      authorization_endpoint: 'https://github.com/login/oauth/authorize',
      token_endpoint: 'https://github.com/login/oauth/access_token',
      userinfo_endpoint: 'https://api.github.com/user',
      jwks_uri: null,
      scopes: ['read:user', 'user:email'],
      claim_mapping: { username: 'login', email: 'email' },
      icon_ref: 'github',
    })

    expect(updated.protocol).toBe('oauth2')
    expect(updated.display_name).toBe('GitHub')
    expect(updated.scopes_input).toContain('read:user')
    expect(updated.icon_ref).toBe('github')
  })
})
