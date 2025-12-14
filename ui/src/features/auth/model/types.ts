export interface LoginResponse {
  // Case 1: Success
  access_token?: string

  // Case 2: Challenge (MFA, etc)
  status?: 'challenge' | 'redirect'
  challenge_page?: string

  // Case 3: OIDC Redirect
  url?: string
}

export interface ExecutionResponse {
  session_id: string
  execution: ExecutionResult
}

// Matches your Rust #[serde(tag = "type", content = "payload")]
export type ExecutionResult =
  | { type: 'Challenge'; payload: ChallengePayload }
  | { type: 'Success'; payload: SuccessPayload }
  | { type: 'Failure'; payload: FailurePayload }

export interface ChallengePayload {
  screen_id: string // e.g. "username_password"
  context: Record<string, any>
}

export interface SuccessPayload {
  redirect_url: string
}

export interface FailurePayload {
  reason: string
}
