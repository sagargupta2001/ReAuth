import { Loader2 } from 'lucide-react'

import type { AuthScreenProps } from '@/entities/auth/model/screenTypes'
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
      ? `${resumePath}?realm=${encodeURIComponent(realm ?? 'master')}&resume_token=${encodeURIComponent(
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
  const resendUrl =
    actionType === 'reset_credentials'
      ? `/forgot-password?realm=${encodeURIComponent(realm ?? 'master')}`
      : null

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      <div className="text-sm font-medium text-foreground">{message}</div>
      <p className="text-xs text-muted-foreground">
        You can keep this page open or come back after completing the step.
      </p>
      {resumeToken ? (
        <div className="w-full max-w-sm rounded-md border border-dashed border-muted-foreground/30 bg-muted/30 p-3 text-xs text-muted-foreground">
          <div className="font-medium text-foreground">Recovery token</div>
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
          Continue reset
        </a>
      ) : null}
      {resendUrl ? (
        <Button variant="secondary" size="sm" asChild>
          <a href={resendUrl}>{isExpired ? 'Request new token' : 'Resend recovery email'}</a>
        </Button>
      ) : null}
    </div>
  )
}
