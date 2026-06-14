import { useState } from 'react'

import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/alert-dialog'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Switch } from '@/components/switch'
import {
  useTelemetryClearLogs,
  useTelemetryClearTraces,
} from '@/features/observability/api/useTelemetryCleanup'
import { useIncludeSpansPreference } from '@/features/observability/lib/observabilityPreferences'
import { useCurrentRealm } from '@/features/realm/api/useRealm'
import { useRealmPasskeyAnalytics } from '@/features/realm/api/useRealmPasskeyAnalytics'
import { RealmSettingsCard } from '@/features/realm/components/RealmSettingsCard'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function ObservabilitySettingsPage() {
  const { t } = useTranslation('logs')
  useHashScrollHighlight()

  const clearLogs = useTelemetryClearLogs()
  const clearTraces = useTelemetryClearTraces()
  const { includeSpans, setIncludeSpans } = useIncludeSpansPreference()
  const { data: realm } = useCurrentRealm()
  const { data: passkeyAnalytics } = useRealmPasskeyAnalytics(realm?.id, 24)

  const [logsOpen, setLogsOpen] = useState(false)
  const [logsInput, setLogsInput] = useState('')
  const [tracesOpen, setTracesOpen] = useState(false)
  const [tracesInput, setTracesInput] = useState('')

  return (
    <div className="max-w-4xl space-y-6 p-12">
      <RealmSettingsCard
        id="telemetry-settings"
        title="Telemetry"
        description="Configure log collection behavior and cleanup actions."
        bodyClassName="space-y-4"
      >
        <div
          id="telemetry-include-spans"
          className="border-border/40 bg-background/60 flex flex-wrap items-center justify-between gap-3 rounded-lg border px-4 py-3"
        >
          <div className="space-y-1">
            <div className="text-sm font-medium">{t('LOGS_EXPLORER.INCLUDE_SPANS')}</div>
            <p className="text-muted-foreground text-xs">
              Show trace span entries alongside logs in the explorer.
            </p>
          </div>
          <Switch
            checked={includeSpans}
            onCheckedChange={(checked) => {
              setIncludeSpans(checked)
              toast.success('Settings updated')
            }}
            aria-label={t('LOGS_EXPLORER.INCLUDE_SPANS')}
          />
        </div>

        <div
          id="logs-cleanup"
          className="border-destructive/30 bg-destructive/5 rounded-lg border p-4"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="space-y-1">
              <div className="text-destructive flex items-center gap-2 text-sm font-semibold">
                <AlertTriangle className="h-4 w-4" />
                {t('LOGS_CLEANUP.TITLE')}
              </div>
              <p className="text-muted-foreground text-xs">{t('LOGS_CLEANUP.DESC')}</p>
            </div>
            <AlertDialog
              open={logsOpen}
              onOpenChange={(open) => {
                setLogsOpen(open)
                if (!open) setLogsInput('')
              }}
            >
              <AlertDialogTrigger asChild>
                <Button variant="destructive">{t('LOGS_CLEANUP.CLEAR_ALL')}</Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>{t('LOGS_CLEANUP.CONFIRM_TITLE')}</AlertDialogTitle>
                  <AlertDialogDescription>{t('LOGS_CLEANUP.CONFIRM_DESC')}</AlertDialogDescription>
                </AlertDialogHeader>
                <div className="space-y-2">
                  <Input
                    placeholder={t('LOGS_CLEANUP.CONFIRM_PLACEHOLDER')}
                    value={logsInput}
                    onChange={(event) => setLogsInput(event.target.value)}
                  />
                  <p className="text-muted-foreground text-xs">
                    {t('LOGS_CLEANUP.CONFIRM_HELPER')}
                  </p>
                </div>
                <AlertDialogFooter>
                  <AlertDialogCancel>{t('LOGS_CLEANUP.CANCEL')}</AlertDialogCancel>
                  <AlertDialogAction
                    className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                    onClick={() => {
                      clearLogs.mutate(undefined, {
                        onSuccess: () => {
                          setLogsInput('')
                        },
                      })
                    }}
                    disabled={logsInput.trim() !== 'CLEAR' || clearLogs.isPending}
                  >
                    {t('LOGS_CLEANUP.CONFIRM_ACTION')}
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
          <p className="text-muted-foreground mt-2 text-xs">{t('LOGS_CLEANUP.CONFIRM_HELPER')}</p>
        </div>
      </RealmSettingsCard>

      <RealmSettingsCard
        id="trace-settings"
        title="Trace"
        description="Manage trace cleanup actions and retention."
      >
        <div
          id="traces-cleanup"
          className="border-destructive/30 bg-destructive/5 rounded-lg border p-4"
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="space-y-1">
              <div className="text-destructive flex items-center gap-2 text-sm font-semibold">
                <AlertTriangle className="h-4 w-4" />
                {t('TRACES_CLEANUP.TITLE')}
              </div>
              <p className="text-muted-foreground text-xs">{t('TRACES_CLEANUP.DESC')}</p>
            </div>
            <AlertDialog
              open={tracesOpen}
              onOpenChange={(open) => {
                setTracesOpen(open)
                if (!open) setTracesInput('')
              }}
            >
              <AlertDialogTrigger asChild>
                <Button variant="destructive">{t('TRACES_CLEANUP.CLEAR_ALL')}</Button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>{t('TRACES_CLEANUP.CONFIRM_TITLE')}</AlertDialogTitle>
                  <AlertDialogDescription>
                    {t('TRACES_CLEANUP.CONFIRM_DESC')}
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <div className="space-y-2">
                  <Input
                    placeholder={t('TRACES_CLEANUP.CONFIRM_PLACEHOLDER')}
                    value={tracesInput}
                    onChange={(event) => setTracesInput(event.target.value)}
                  />
                  <p className="text-muted-foreground text-xs">
                    {t('TRACES_CLEANUP.CONFIRM_HELPER')}
                  </p>
                </div>
                <AlertDialogFooter>
                  <AlertDialogCancel>{t('TRACES_CLEANUP.CANCEL')}</AlertDialogCancel>
                  <AlertDialogAction
                    className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                    onClick={() => {
                      clearTraces.mutate(undefined, {
                        onSuccess: () => {
                          setTracesInput('')
                        },
                      })
                    }}
                    disabled={tracesInput.trim() !== 'CLEAR' || clearTraces.isPending}
                  >
                    {t('TRACES_CLEANUP.CONFIRM_ACTION')}
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
          <p className="text-muted-foreground mt-2 text-xs">{t('TRACES_CLEANUP.CONFIRM_HELPER')}</p>
        </div>
      </RealmSettingsCard>

      <RealmSettingsCard
        id="passkey-observability"
        title="Passkeys"
        description="Inspect passkey enrollment and assertion diagnostics from the last 24 hours."
        bodyClassName="space-y-4"
      >
        <div className="grid gap-3 md:grid-cols-3">
          <div className="border-border/40 bg-background/60 rounded-lg border px-4 py-3">
            <div className="text-muted-foreground text-xs">Credentials</div>
            <div className="text-xl font-semibold">{passkeyAnalytics?.credentials_total ?? 0}</div>
            <div className="text-muted-foreground text-xs">
              +{passkeyAnalytics?.credentials_created_last_7d ?? 0} new in 7d
            </div>
          </div>
          <div className="border-border/40 bg-background/60 rounded-lg border px-4 py-3">
            <div className="text-muted-foreground text-xs">Assertions Succeeded</div>
            <div className="text-xl font-semibold">
              {passkeyAnalytics?.outcomes.assertion_success ?? 0}
            </div>
            <div className="text-muted-foreground text-xs">last 24h</div>
          </div>
          <div className="border-border/40 bg-background/60 rounded-lg border px-4 py-3">
            <div className="text-muted-foreground text-xs">Pending Challenges</div>
            <div className="text-xl font-semibold">
              {passkeyAnalytics?.challenges.pending_total ?? 0}
            </div>
            <div className="text-muted-foreground text-xs">
              expired: {passkeyAnalytics?.challenges.pending_expired ?? 0}
            </div>
          </div>
        </div>

        <div className="border-border/40 bg-background/60 rounded-lg border px-4 py-3">
          <div className="text-sm font-medium">Failure Signals</div>
          <p className="text-muted-foreground mt-1 text-xs">
            Invalid signatures, challenge mismatches, and counter regressions in last 24h.
          </p>
          <div className="mt-3 grid gap-2 text-sm md:grid-cols-2">
            <div>
              Assertion invalid signature:{' '}
              {passkeyAnalytics?.outcomes.assertion_invalid_signature ?? 0}
            </div>
            <div>
              Assertion challenge mismatch:{' '}
              {passkeyAnalytics?.outcomes.assertion_challenge_mismatch ?? 0}
            </div>
            <div>
              Assertion counter regression:{' '}
              {passkeyAnalytics?.outcomes.assertion_counter_regression ?? 0}
            </div>
            <div>
              Enrollment challenge mismatch:{' '}
              {passkeyAnalytics?.outcomes.enrollment_challenge_mismatch ?? 0}
            </div>
          </div>
        </div>

        <div className="border-border/40 bg-background/60 rounded-lg border px-4 py-3">
          <div className="text-sm font-medium">Recent Passkey Failures</div>
          <div className="mt-2 space-y-2">
            {(passkeyAnalytics?.recent_failures ?? []).length === 0 ? (
              <p className="text-muted-foreground text-xs">No recent passkey failures.</p>
            ) : (
              (passkeyAnalytics?.recent_failures ?? []).map((event) => (
                <div
                  key={`${event.action}-${event.created_at}-${event.target_id ?? 'none'}`}
                  className="border-border/40 rounded border px-3 py-2 text-xs"
                >
                  <div className="font-medium">{event.action}</div>
                  <div className="text-muted-foreground">
                    {new Date(event.created_at).toLocaleString()}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </RealmSettingsCard>
    </div>
  )
}

export default ObservabilitySettingsPage
