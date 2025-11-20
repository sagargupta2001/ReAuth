export const OIDC_CONFIG = {
  clientId: 'reauth-admin', // Must match the seeded client
  redirectUri: window.location.origin, // e.g. http://localhost:5173
  scope: 'openid profile email',
  responseType: 'code',
  codeChallengeMethod: 'S256',
}

export const PKCE_STORAGE_KEY = 'reauth_pkce_verifier'
