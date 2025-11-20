export const OIDC_CONFIG = {
  realm: 'master',
  clientId: 'reauth-admin',
  redirectUri: window.location.origin,
  scope: 'openid profile email',
  responseType: 'code',
  codeChallengeMethod: 'S256',
}

export const PKCE_STORAGE_KEY = 'reauth_pkce_verifier'
