import { BaseAuthFlowExecutor } from '@/features/auth/components/AuthFlowExecutor.tsx'

export function ForgotPasswordPage() {
  return <BaseAuthFlowExecutor flowPath="reset" />
}
