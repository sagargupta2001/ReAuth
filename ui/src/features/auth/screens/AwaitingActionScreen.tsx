import { useEffect, useMemo, useRef, useState } from 'react'
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
  const activeRealm = realm ?? 'master'
  const resumeRedirectUrl = useMemo(() => {
    if (!resumePath) return null
    const base = `/#${resumePath}`
    const params = new URLSearchParams()
    params.set('realm', activeRealm)
    const joiner = resumePath.includes('?') ? '&' : '?'
    return `${base}${joiner}${params.toString()}`
  }, [resumePath, activeRealm])
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
  const canResend =
    Boolean(resumeToken) &&
    (actionType === 'reset_credentials' || actionType === 'email_verify')
  const [resendStatus, setResendStatus] = useState<'idle' | 'sending' | 'sent' | 'error'>(
    'idle',
  )
  const [autoStatus, setAutoStatus] = useState<'idle' | 'consumed' | 'expired' | 'error'>('idle')
  const pollDelayRef = useRef(2000)
  const pollTimeoutRef = useRef<number | null>(null)
  const resendLabel = isExpired
    ? 'Request new code'
    : actionType === 'email_verify'
      ? 'Resend verification email'
      : 'Resend recovery email'

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

  useEffect(() => {
    if (!resumeToken || !resumePath) return
    if (autoStatus === 'consumed' || autoStatus === 'expired') return
    let cancelled = false

    const poll = async () => {
      try {
        const response = await authApi.actionStatus(activeRealm, resumeToken)
        if (cancelled) return
        if (response.status === 'consumed') {
          setAutoStatus('consumed')
          if (resumeRedirectUrl) {
            window.setTimeout(() => {
              window.location.href = resumeRedirectUrl
            }, 1200)
          }
          return
        }
        if (response.status === 'expired') {
          setAutoStatus('expired')
          return
        }
        if (autoStatus !== 'idle') {
          setAutoStatus('idle')
        }
      } catch {
        if (!cancelled) {
          setAutoStatus('error')
        }
      }

      const nextDelay = Math.min(10000, Math.round(pollDelayRef.current * 1.5))
      pollDelayRef.current = nextDelay
      pollTimeoutRef.current = window.setTimeout(poll, pollDelayRef.current)
    }

    pollTimeoutRef.current = window.setTimeout(poll, pollDelayRef.current)
    return () => {
      cancelled = true
      if (pollTimeoutRef.current) {
        window.clearTimeout(pollTimeoutRef.current)
      }
    }
  }, [resumeToken, resumePath, activeRealm, resumeRedirectUrl, autoStatus])

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      <div className="text-sm font-medium text-foreground">{message}</div>
      <p className="text-xs text-muted-foreground">
        You can keep this page open or come back after completing the step.
      </p>
      {expiresAt ? (
        <div className="text-xs text-muted-foreground">Expires: {expiresAt}</div>
      ) : null}
      {expiresInMinutes != null && !isExpired ? (
        <div className="text-xs text-muted-foreground">
          Expires in ~{expiresInMinutes} min
        </div>
      ) : null}
      {isExpired ? <div className="text-xs text-muted-foreground">Token expired.</div> : null}
      {autoStatus === 'consumed' ? (
        <div className="text-xs text-muted-foreground">Recovery confirmed, redirecting…</div>
      ) : null}
      {autoStatus === 'expired' ? (
        <div className="text-xs text-muted-foreground">Token expired. Request a new one.</div>
      ) : null}
      {autoStatus === 'error' ? (
        <div className="text-xs text-muted-foreground">Waiting for confirmation…</div>
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
