import { useRef, useState } from 'react'

import type { LogEntry } from '@/entities/log/model/types'

const MAX_LOGS = 1000 // Keep only the latest 1000 logs in state

export function useLogStream() {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const ws = useRef<WebSocket | null>(null)

  // This function is for your "Toggle Real-time" button
  const toggleConnection = () => {
    if (ws.current) {
      ws.current.close()
      ws.current = null
      setIsConnected(false)
    } else {
      const socket = new WebSocket(`ws://${window.location.host}/api/logs/ws`)

      socket.onopen = () => {
        console.log('✅ Log stream connected')
        setIsConnected(true)
      }

      socket.onmessage = (event) => {
        const newLog = JSON.parse(event.data)
        setLogs((prevLogs) => {
          // Add new log to the start, and cap the array size
          const nextLogs = [newLog, ...prevLogs]
          if (nextLogs.length > MAX_LOGS) {
            return nextLogs.slice(0, MAX_LOGS)
          }
          return nextLogs
        })
      }

      socket.onclose = () => {
        console.log('❌ Log stream disconnected')
        setIsConnected(false)
        ws.current = null
      }

      ws.current = socket
    }
  }

  return { logs, isConnected, toggleConnection }
}
