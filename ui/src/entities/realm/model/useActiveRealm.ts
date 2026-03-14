import { useParams } from 'react-router-dom'

import { DEFAULT_REALM } from '@/shared/config/auth.ts'
import { isNonRealmRoute, resolveRealmFromLocation } from '@/shared/lib/realm'

export function useActiveRealm() {
  const { realm } = useParams<{ realm: string }>()
  if (realm && !isNonRealmRoute(realm)) return realm
  return resolveRealmFromLocation(DEFAULT_REALM)
}
