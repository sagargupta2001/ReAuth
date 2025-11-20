import { OIDC_CONFIG } from '@/shared/config/oidc'

export interface AuthorizeResponse {
  status: 'challenge' | 'redirect'
  // If challenge
  challenge_page?: string
  // If redirect (end of flow)
  url?: string
}

export interface TokenResponse {
  access_token: string
  token_type: string
  expires_in: number
}

export const oidcApi = {
  /**
   * Call /authorize to start the flow or check status
   */
  authorize: async (codeChallenge: string) => {
    const params = new URLSearchParams({
      client_id: OIDC_CONFIG.clientId,
      redirect_uri: OIDC_CONFIG.redirectUri,
      response_type: OIDC_CONFIG.responseType,
      scope: OIDC_CONFIG.scope,
      code_challenge: codeChallenge,
      code_challenge_method: OIDC_CONFIG.codeChallengeMethod,
    })

    const res = await fetch(`/api/oidc/authorize?${params.toString()}`, {
      method: 'GET',
    })

    if (!res.ok) throw new Error('Failed to initialize OIDC flow')
    return res.json() as Promise<AuthorizeResponse>
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

    const res = await fetch('/api/oidc/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: params,
    })

    if (!res.ok) {
      const err = await res.text()
      throw new Error(`Token exchange failed: ${err}`)
    }

    return res.json() as Promise<TokenResponse>
  },
}
