import { create } from 'zustand'

import type { LogEntry } from '@/entities/log/model/types.ts'

const MAX_LOGS = 1000 // Keep only the latest 1000 logs
const RECONNECT_BASE_DELAY_MS = 1000
const RECONNECT_MAX_DELAY_MS = 10_000

interface LogState {
  logs: LogEntry[]
  isConnected: boolean
  ws: WebSocket | null
  reconnectAttempts: number
  reconnectTimer: number | null
  shouldReconnect: boolean
  connect: () => void
  disconnect: () => void
  addLog: (log: LogEntry) => void
  clearLogs: () => void
}

export const useLogStore = create<LogState>((set, get) => ({
  logs: [],
  isConnected: false,
  ws: null,
  reconnectAttempts: 0,
  reconnectTimer: null,
  shouldReconnect: true,

  /**
   * Connects to the WebSocket log stream.
   * Does nothing if already connected.
   */
  connect: () => {
    // Don't connect if we already have a WebSocket
    if (get().ws) return
    if (get().reconnectTimer) {
      window.clearTimeout(get().reconnectTimer!)
      set({ reconnectTimer: null })
    }
    set({ shouldReconnect: true })

    const socket = new WebSocket(`ws://${window.location.host}/api/logs/ws`)

    socket.onopen = () => {
      console.log('✅ Log stream connected')
      set({ isConnected: true, reconnectAttempts: 0 })
    }

    socket.onmessage = (event) => {
      const newLog = JSON.parse(event.data)
      get().addLog(newLog) // Call the store's own action
    }

    socket.onclose = () => {
      console.log('❌ Log stream disconnected')
      set({ isConnected: false, ws: null })
      if (!get().shouldReconnect) return

      const attempt = get().reconnectAttempts + 1
      const delay = Math.min(RECONNECT_BASE_DELAY_MS * 2 ** (attempt - 1), RECONNECT_MAX_DELAY_MS)
      const timer = window.setTimeout(() => {
        set({ reconnectTimer: null })
        get().connect()
      }, delay)
      set({ reconnectAttempts: attempt, reconnectTimer: timer })
    }

    socket.onerror = (err) => {
      console.error('WebSocket Error:', err)
      set({ isConnected: false, ws: null })
      socket.close()
    }

    set({ ws: socket })
  },

  /**
   * Disconnects from the WebSocket log stream.
   */
  disconnect: () => {
    const timer = get().reconnectTimer
    if (timer) {
      window.clearTimeout(timer)
    }
    set({ shouldReconnect: false, reconnectTimer: null, reconnectAttempts: 0 })
    get().ws?.close()
    set({ isConnected: false, ws: null })
  },

  /**
   * Adds a new log entry to the state.
   */
  addLog: (log) => {
    set((state) => {
      const nextLogs = [log, ...state.logs]
      // Cap the array size to prevent memory leaks
      if (nextLogs.length > MAX_LOGS) {
        return { logs: nextLogs.slice(0, MAX_LOGS) }
      }
      return { logs: nextLogs }
    })
  },

  /**
   * Clears all logs from the state.
   */
  clearLogs: () => set({ logs: [] }),
}))
