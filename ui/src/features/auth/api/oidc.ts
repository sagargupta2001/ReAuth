import { OIDC_CONFIG } from '@/shared/config/oidc'

export interface TokenResponse {
  access_token: string
  token_type: string
  expires_in: number
}

export const oidcApi = {
  /**
   * Constructs the OIDC Authorize URL.
   * This is a navigation URL, not an API endpoint we fetch.
   */
  getAuthorizeUrl: (codeChallenge: string) => {
    const params = new URLSearchParams({
      client_id: OIDC_CONFIG.clientId,
      redirect_uri: OIDC_CONFIG.redirectUri,
      response_type: OIDC_CONFIG.responseType,
      scope: OIDC_CONFIG.scope,
      code_challenge: codeChallenge,
      code_challenge_method: OIDC_CONFIG.codeChallengeMethod,
    })

    return `/api/realms/${OIDC_CONFIG.realm}/oidc/authorize?${params.toString()}`
  },

  /**
   * Exchange the Auth Code for a Token
   */
  exchangeToken: async (code: string, verifier: string) => {
    const params = new URLSearchParams()
    params.append('grant_type', 'authorization_code')
    params.append('code', code)
    params.append('redirect_uri', OIDC_CONFIG.redirectUri)
    params.append('client_id', OIDC_CONFIG.clientId)
    params.append('code_verifier', verifier)

    // OIDC spec requires 'application/x-www-form-urlencoded'
    const res = await fetch(`/api/realms/${OIDC_CONFIG.realm}/oidc/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params,
    })

    if (!res.ok) {
      const errorData = await res.json().catch(() => ({}))
      throw new Error(errorData.error || `Token exchange failed: ${res.statusText}`)
    }

    return (await res.json()) as Promise<TokenResponse>
  },
}
