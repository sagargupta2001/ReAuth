import { useState } from 'react'

import { Play, StopCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import type { LogEntry } from '@/entities/log/model/types.ts'

import { LogRowDetail } from './components/LogRowDetail'
import { LogTable } from './components/LogTable'
import { useLogStream } from './hooks/useLogStream'

export function LogViewerWidget() {
  const { t } = useTranslation('logs')
  const { logs, isConnected, connect, disconnect } = useLogStream()
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null)

  return (
    <div className="absolute inset-0 flex flex-col p-4">
      <div className="mb-4 flex shrink-0 justify-end">
        <Button variant="outline" onClick={isConnected ? disconnect : connect}>
          {isConnected ? (
            <>
              <StopCircle className="mr-2 h-4 w-4 text-red-500" />{' '}
              {t('LOG_VIEWER_WIDGET.STOP_STREAM_BTN_TEXT')}
            </>
          ) : (
            <>
              <Play className="mr-2 h-4 w-4 text-green-500" />{' '}
              {t('LOG_VIEWER_WIDGET.START_STREAM_BTN_TEXT')}
            </>
          )}
        </Button>
      </div>

      <div className="relative min-h-0 flex-1">
        <LogTable logs={logs} onRowClick={setSelectedLog} />
      </div>

      <LogRowDetail log={selectedLog} onOpenChange={(open) => !open && setSelectedLog(null)} />
    </div>
  )
}
