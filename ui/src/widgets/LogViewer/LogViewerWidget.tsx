import { Play, StopCircle } from 'lucide-react'

import { Button } from '@/components/button'

import { LogTable } from './components/LogTable'
import { useLogStream } from './hooks/useLogStream'

export function LogViewerWidget() {
  const { logs, isConnected, toggleConnection } = useLogStream()

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <Button variant="outline" onClick={toggleConnection}>
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
      <LogTable logs={logs} />
    </div>
  )
}
