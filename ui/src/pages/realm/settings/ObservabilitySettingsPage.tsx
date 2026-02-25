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
import { useTelemetryClearLogs, useTelemetryClearTraces } from '@/features/observability/api/useTelemetryCleanup'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function ObservabilitySettingsPage() {
  const { t } = useTranslation('logs')
  useHashScrollHighlight()

  const clearLogs = useTelemetryClearLogs()
  const clearTraces = useTelemetryClearTraces()

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

      <Card id="logs-cleanup" className="border-destructive/20">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertTriangle className="h-4 w-4 text-destructive" />
            {t('LOGS_CLEANUP.TITLE')}
          </CardTitle>
          <p className="text-sm text-muted-foreground">{t('LOGS_CLEANUP.DESC')}</p>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center justify-between gap-3">
          <div className="text-xs text-muted-foreground">
            {t('LOGS_CLEANUP.CONFIRM_HELPER')}
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
                <p className="text-xs text-muted-foreground">{t('LOGS_CLEANUP.CONFIRM_HELPER')}</p>
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
        </CardContent>
      </Card>

      <Card id="traces-cleanup" className="border-destructive/20">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertTriangle className="h-4 w-4 text-destructive" />
            {t('TRACES_CLEANUP.TITLE')}
          </CardTitle>
          <p className="text-sm text-muted-foreground">{t('TRACES_CLEANUP.DESC')}</p>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center justify-between gap-3">
          <div className="text-xs text-muted-foreground">
            {t('TRACES_CLEANUP.CONFIRM_HELPER')}
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
                <AlertDialogDescription>{t('TRACES_CLEANUP.CONFIRM_DESC')}</AlertDialogDescription>
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
        </CardContent>
      </Card>
    </div>
  )
}

export default ObservabilitySettingsPage
