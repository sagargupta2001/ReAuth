import { useState } from 'react'

import { Play, StopCircle } from 'lucide-react'

import { Button } from '@/components/button'
import type { LogEntry } from '@/entities/log/model/types.ts'

import { LogRowDetail } from './components/LogRowDetail'
import { LogTable } from './components/LogTable'
import { useLogStream } from './hooks/useLogStream'

export function LogViewerWidget() {
  const { logs, isConnected, connect, disconnect } = useLogStream()
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null)

  return (
    <div className="flex h-full flex-col">
      <div className="mb-4 flex justify-end">
        <Button variant="outline" onClick={isConnected ? disconnect : connect}>
          {isConnected ? (
            <>
              <StopCircle className="mr-2 h-4 w-4 text-red-500" /> Stop Stream
            </>
          ) : (
            <>
              <Play className="mr-2 h-4 w-4 text-green-500" /> Start Stream
            </>
          )}
        </Button>
      </div>

      <div className="flex-1">
        <LogTable logs={logs} onRowClick={setSelectedLog} />
      </div>

      <LogRowDetail
        log={selectedLog}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedLog(null)
          }
        }}
      />
    </div>
  )
}
