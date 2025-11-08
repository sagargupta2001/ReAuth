import { useEffect } from 'react'

import { useAppStore } from '@/shared/store/appStore'

export const useStoreHydration = () => {
  const hasHydrated = useAppStore((s) => s.hasHydrated)
  const setHydrated = useAppStore((s) => s.setHydrated)

  useEffect(() => {
    // Simulate async store rehydration (e.g., persisted Zustand stores or cookie restoration)
    // If your app doesn’t persist state, you can just call it immediately.
    const timeout = setTimeout(() => setHydrated(true), 0)
    return () => clearTimeout(timeout)
  }, [setHydrated])

  return hasHydrated
}
