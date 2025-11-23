import { useParams } from 'react-router-dom'

import { DEFAULT_REALM } from '@/shared/config/auth.ts'

export function useActiveRealm() {
  const { realm } = useParams<{ realm: string }>()
  // Default to 'DEFAULT_REALM' if something goes wrong or we are at root
  return realm || DEFAULT_REALM
}
