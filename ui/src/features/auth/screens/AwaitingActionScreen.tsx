import { useState } from 'react'
import { Loader2 } from 'lucide-react'

import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'
import { authApi } from '@/features/auth/api/authApi'
import { Button } from '@/shared/ui/button.tsx'

export function AwaitingActionScreen({ context, realm }: AuthScreenProps) {
  const message =
    (typeof context.message === 'string' && context.message) ||
    (typeof context.description === 'string' && context.description) ||
    'Waiting for verification...'
  const resumeToken =
    typeof context.resume_token === 'string' ? context.resume_token : null
  const resumePath =
    typeof context.resume_path === 'string' ? context.resume_path : null
  const actionType =
    typeof context.action_type === 'string' ? context.action_type : null
  const resumeUrl =
    resumeToken && resumePath
      ? `/#${resumePath}?realm=${encodeURIComponent(realm ?? 'master')}&resume_token=${encodeURIComponent(
          resumeToken,
        )}`
      : null
  const expiresAt =
    typeof context.expires_at === 'string'
      ? context.expires_at
      : context.expires_at instanceof Date
        ? context.expires_at.toISOString()
        : null
  const expiresAtDate = expiresAt ? new Date(expiresAt) : null
  const expiresInMinutes =
    expiresAtDate != null
      ? Math.max(0, Math.ceil((expiresAtDate.getTime() - Date.now()) / 60000))
      : null
  const isExpired = expiresAtDate ? expiresAtDate.getTime() <= Date.now() : false
  const tokenLabel = actionType === 'email_verify' ? 'Verification code' : 'Recovery token'
  const canResend =
    Boolean(resumeToken) &&
    (actionType === 'reset_credentials' || actionType === 'email_verify')
  const [resendStatus, setResendStatus] = useState<'idle' | 'sending' | 'sent' | 'error'>(
    'idle',
  )
  const resendLabel = isExpired
    ? 'Request new code'
    : actionType === 'email_verify'
      ? 'Resend verification email'
      : 'Resend recovery email'

  const activeRealm = realm ?? 'master'

  const handleResend = async () => {
    if (!resumeToken) return
    setResendStatus('sending')
    try {
      await authApi.resendAction(activeRealm, resumeToken)
      setResendStatus('sent')
    } catch (error) {
      console.error('[AwaitingAction] Resend failed', error)
      setResendStatus('error')
    }
  }

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      <div className="text-sm font-medium text-foreground">{message}</div>
      <p className="text-xs text-muted-foreground">
        You can keep this page open or come back after completing the step.
      </p>
      {resumeToken ? (
        <div className="w-full max-w-sm rounded-md border border-dashed border-muted-foreground/30 bg-muted/30 p-3 text-xs text-muted-foreground">
          <div className="font-medium text-foreground">{tokenLabel}</div>
          <div className="mt-1 break-all font-mono">{resumeToken}</div>
          {expiresAt ? <div className="mt-1">Expires: {expiresAt}</div> : null}
          {expiresInMinutes != null && !isExpired ? (
            <div className="mt-1">Expires in ~{expiresInMinutes} min</div>
          ) : null}
          {isExpired ? <div className="mt-1">Token expired.</div> : null}
        </div>
      ) : null}
      {resumeUrl ? (
        <a className="text-xs font-medium text-primary underline" href={resumeUrl}>
          {actionType === 'email_verify' ? 'Continue verification' : 'Continue reset'}
        </a>
      ) : null}
      {canResend ? (
        <>
          <Button
            variant="secondary"
            size="sm"
            disabled={resendStatus === 'sending'}
            onClick={handleResend}
          >
            {resendStatus === 'sending' ? 'Sending…' : resendLabel}
          </Button>
          {resendStatus === 'sent' ? (
            <div className="text-xs text-muted-foreground">Email sent.</div>
          ) : null}
          {resendStatus === 'error' ? (
            <div className="text-xs text-destructive">Unable to resend email.</div>
          ) : null}
        </>
      ) : null}
    </div>
  )
}
