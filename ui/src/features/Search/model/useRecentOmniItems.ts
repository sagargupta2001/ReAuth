import { useCallback, useMemo, useState } from 'react'

const STORAGE_KEY = 'reauth.omni.recent'
const MAX_RECENT = 25

type RecentEntry = {
  id: string
  ts: number
}

function readRecent(): RecentEntry[] {
  if (typeof window === 'undefined') return []
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as RecentEntry[]
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

function writeRecent(entries: RecentEntry[]) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries))
}

export function useRecentOmniItems() {
  const [recent, setRecent] = useState<RecentEntry[]>(() => readRecent())

  const recordSelection = useCallback((id: string) => {
    setRecent((prev) => {
      const next = [{ id, ts: Date.now() }, ...prev.filter((entry) => entry.id !== id)]
      const trimmed = next.slice(0, MAX_RECENT)
      writeRecent(trimmed)
      return trimmed
    })
  }, [])

  const recencyMap = useMemo(() => {
    const map = new Map<string, number>()
    recent.forEach((entry, index) => {
      map.set(entry.id, MAX_RECENT - index)
    })
    return map
  }, [recent])

  return {
    recordSelection,
    recencyMap,
  }
}
