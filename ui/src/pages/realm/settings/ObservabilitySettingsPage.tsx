import { useState } from 'react'

import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

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
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { Input } from '@/components/input'
import { Switch } from '@/components/switch'
import { useTelemetryClearLogs, useTelemetryClearTraces } from '@/features/observability/api/useTelemetryCleanup'
import { useIncludeSpansPreference } from '@/features/observability/lib/observabilityPreferences'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'
import { toast } from 'sonner'

export function ObservabilitySettingsPage() {
  const { t } = useTranslation('logs')
  useHashScrollHighlight()

  const clearLogs = useTelemetryClearLogs()
  const clearTraces = useTelemetryClearTraces()
  const { includeSpans, setIncludeSpans } = useIncludeSpansPreference()

  const [logsOpen, setLogsOpen] = useState(false)
  const [logsInput, setLogsInput] = useState('')
  const [tracesOpen, setTracesOpen] = useState(false)
  const [tracesInput, setTracesInput] = useState('')

  return (
    <div className="max-w-4xl space-y-6 p-12">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Observability Settings</h1>
        <p className="text-sm text-muted-foreground">
          Manage log retention actions and trace cleanup for this realm.
        </p>
      </div>

      <Card id="telemetry-settings">
        <CardHeader>
          <CardTitle className="text-base">Telemetry</CardTitle>
          <p className="text-sm text-muted-foreground">
            Configure log collection behavior and cleanup actions.
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <div
            id="telemetry-include-spans"
            className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border/40 bg-background/60 px-4 py-3"
          >
            <div className="space-y-1">
              <div className="text-sm font-medium">{t('LOGS_EXPLORER.INCLUDE_SPANS')}</div>
              <p className="text-xs text-muted-foreground">
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
            className="rounded-lg border border-destructive/30 bg-destructive/5 p-4"
          >
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="space-y-1">
                <div className="flex items-center gap-2 text-sm font-semibold text-destructive">
                  <AlertTriangle className="h-4 w-4" />
                  {t('LOGS_CLEANUP.TITLE')}
                </div>
                <p className="text-xs text-muted-foreground">{t('LOGS_CLEANUP.DESC')}</p>
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
                    <p className="text-xs text-muted-foreground">
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
            <p className="mt-2 text-xs text-muted-foreground">
              {t('LOGS_CLEANUP.CONFIRM_HELPER')}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card id="trace-settings">
        <CardHeader>
          <CardTitle className="text-base">Trace</CardTitle>
          <p className="text-sm text-muted-foreground">
            Manage trace cleanup actions and retention.
          </p>
        </CardHeader>
        <CardContent>
          <div
            id="traces-cleanup"
            className="rounded-lg border border-destructive/30 bg-destructive/5 p-4"
          >
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="space-y-1">
                <div className="flex items-center gap-2 text-sm font-semibold text-destructive">
                  <AlertTriangle className="h-4 w-4" />
                  {t('TRACES_CLEANUP.TITLE')}
                </div>
                <p className="text-xs text-muted-foreground">{t('TRACES_CLEANUP.DESC')}</p>
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
                    <p className="text-xs text-muted-foreground">
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
            <p className="mt-2 text-xs text-muted-foreground">
              {t('TRACES_CLEANUP.CONFIRM_HELPER')}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

export default ObservabilitySettingsPage
