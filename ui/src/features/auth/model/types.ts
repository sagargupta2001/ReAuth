export interface LoginResponse {
  // Case 1: Success
  access_token?: string

  // Case 2: Challenge (MFA, etc)
  status?: 'challenge' | 'redirect'
  challenge_page?: string

  // Case 3: OIDC Redirect
  url?: string
}
