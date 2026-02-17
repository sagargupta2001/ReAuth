export interface AuthScreenProps {
  /** The data payload from the backend for this step (labels, config, etc) */
  context: Record<string, any>

  /** Function to call when the user submits the form */
  onSubmit: (data: any) => Promise<void>

  /** UI State passed down from the executor */
  isLoading: boolean
  error: string | null
}
