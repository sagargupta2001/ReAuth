import { useLogStore } from '@/entities/log/model/logStore.ts'

/**
 * Hook to roles the global log stream state and actions.
 */
export function useLogStream() {
  // Select the state and actions needed from the store
  const logs = useLogStore((state) => state.logs)
  const isConnected = useLogStore((state) => state.isConnected)
  const connect = useLogStore((state) => state.connect)
  const disconnect = useLogStore((state) => state.disconnect)

  // Return them for the component to use
  return { logs, isConnected, connect, disconnect }
}
