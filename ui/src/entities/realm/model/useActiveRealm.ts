import { useParams } from 'react-router-dom'

import { DEFAULT_REALM } from '@/shared/config/auth.ts'

export function useActiveRealm() {
  const { realm } = useParams<{ realm: string }>()
  if (realm) return realm

  try {
    const hash = window.location.hash || ''
    const hashPath = hash.startsWith('#') ? hash.slice(1) : hash
    const normalized = hashPath.startsWith('/') ? hashPath.slice(1) : hashPath
    const segment = normalized.split('/').filter(Boolean)[0]
    if (segment) return segment
  } catch {
    // ignore parsing errors
  }

  // Default to 'DEFAULT_REALM' if something goes wrong or we are at root
  return DEFAULT_REALM
}
