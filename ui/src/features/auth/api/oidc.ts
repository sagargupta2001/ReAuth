import { apiClient } from '@/shared/api/client'
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
  authorize: (codeChallenge: string) => {
    const params = new URLSearchParams({
      client_id: OIDC_CONFIG.clientId,
      redirect_uri: OIDC_CONFIG.redirectUri,
      response_type: OIDC_CONFIG.responseType,
      scope: OIDC_CONFIG.scope,
      code_challenge: codeChallenge,
      code_challenge_method: OIDC_CONFIG.codeChallengeMethod,
    })

    // Use apiClient for automatic error handling and JSON parsing.
    return apiClient.get<AuthorizeResponse>(
      `/api/realms/${OIDC_CONFIG.realm}/oidc/authorize?${params.toString()}`,
    )
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

    // We use raw fetch here because the OIDC spec specifically requires
    // 'application/x-www-form-urlencoded', and our apiClient is built
    // for 'application/json'.
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
