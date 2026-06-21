import { useEffect, useMemo } from 'react'

import { create } from 'zustand'

interface BreadcrumbState {
  /** Label overrides keyed by raw path segment value (e.g. an entity id → name). */
  overrides: Record<string, string>
  setOverrides: (o: Record<string, string>) => void
  clearKeys: (keys: string[]) => void
}

export const useBreadcrumbStore = create<BreadcrumbState>((set) => ({
  overrides: {},
  setOverrides: (o) => set((s) => ({ overrides: { ...s.overrides, ...o } })),
  clearKeys: (keys) =>
    set((s) => {
      const next = { ...s.overrides }
      for (const k of keys) delete next[k]
      return { overrides: next }
    }),
}))

/**
 * Override breadcrumb labels for dynamic path segments — typically resolving an
 * entity id in the URL to a human name. Keys are raw segment values; labels are
 * strings. Overrides are merged on mount and cleared on unmount.
 *
 * @example
 * const { userId } = useParams()
 * useSetBreadcrumb({ [userId ?? '']: user?.name ?? 'User' })
 */
export function useSetBreadcrumb(overrides: Record<string, string>) {
  const setOverrides = useBreadcrumbStore((s) => s.setOverrides)
  const clearKeys = useBreadcrumbStore((s) => s.clearKeys)

  // Drop empty keys/values so a not-yet-loaded name doesn't register noise.
  const clean = useMemo(
    () => Object.fromEntries(Object.entries(overrides).filter(([k, v]) => k && v)),
    // re-derive only when the serialized content changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [JSON.stringify(overrides)],
  )

  useEffect(() => {
    const keys = Object.keys(clean)
    if (keys.length === 0) return
    setOverrides(clean)
    return () => clearKeys(keys)
  }, [clean, setOverrides, clearKeys])
}
