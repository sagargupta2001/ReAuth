export interface AuthScreenProps {
  /** The data payload from the backend for this step (labels, config, etc) */
  context: Record<string, unknown>

  /** Function to call when the user submits the form */
  onSubmit: (data: Record<string, unknown>) => Promise<void>

  /** UI State passed down from the executor */
  isLoading: boolean
  error: string | null
}
