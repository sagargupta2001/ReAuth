/**
 * Represents the response from the Flow Engine (Executor).
 * Matches the JSON output from `src/adapters/web/auth_handler.rs`.
 */
export type AuthExecutionResponse =
  | {
      status: 'challenge'
      challengeName: string // e.g. "login-password", "mfa-otp"
      context: Record<string, unknown> // e.g. { error: "Invalid credentials", username: "admin" }
    }
  | {
      status: 'redirect'
      url: string // e.g. "/" or "https://oidc-client.com/callback?code=..."
    }
  | {
      status: 'awaiting_action'
      challengeName: string // e.g. "awaiting-action"
      context: Record<string, unknown>
    }
  | {
      status: 'failure'
      message: string // e.g. "Account locked"
    }

/**
 * Helper alias if you want to keep naming consistent with older code,
 * but 'AuthExecutionResponse' is more accurate.
 */
export type ExecutionResult = AuthExecutionResponse

/**
 * Helper type for the screen registry to know what props a screen receives.
 */
export interface AuthScreenProps {
  isLoading: boolean
  error: string | null
  context: Record<string, unknown>
  onSubmit: (data: Record<string, unknown>) => void
}
