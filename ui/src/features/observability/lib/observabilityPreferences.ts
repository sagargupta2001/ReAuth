import { useEffect, useState } from 'react'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'

const storageKey = (realm: string) => `reauth:observability:include-spans:${realm}`
const includeSpansEvent = 'reauth:observability:include-spans'

export function readIncludeSpansPreference(realm: string) {
  if (typeof window === 'undefined') return false
  return window.localStorage.getItem(storageKey(realm)) === 'true'
}

export function writeIncludeSpansPreference(realm: string, value: boolean) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(storageKey(realm), String(value))
  window.dispatchEvent(
    new CustomEvent(includeSpansEvent, { detail: { realm, value } }),
  )
}

export function useIncludeSpansPreference() {
  const realm = useActiveRealm()
  const [includeSpans, setIncludeSpans] = useState(() => readIncludeSpansPreference(realm))

  useEffect(() => {
    setIncludeSpans(readIncludeSpansPreference(realm))
  }, [realm])

  useEffect(() => {
    writeIncludeSpansPreference(realm, includeSpans)
  }, [includeSpans, realm])

  useEffect(() => {
    const handleStorage = (event: StorageEvent) => {
      if (event.key !== storageKey(realm)) return
      setIncludeSpans(event.newValue === 'true')
    }
    window.addEventListener('storage', handleStorage)
    return () => window.removeEventListener('storage', handleStorage)
  }, [realm])

  useEffect(() => {
    const handleCustom = (event: Event) => {
      const detail = (event as CustomEvent<{ realm: string; value: boolean }>).detail
      if (!detail || detail.realm !== realm) return
      setIncludeSpans(detail.value)
    }
    window.addEventListener(includeSpansEvent, handleCustom)
    return () => window.removeEventListener(includeSpansEvent, handleCustom)
  }, [realm])

  return { includeSpans, setIncludeSpans }
}
