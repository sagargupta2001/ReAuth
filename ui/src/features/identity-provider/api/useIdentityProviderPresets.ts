import { useQuery } from '@tanstack/react-query'

import type { IdentityProviderPreset } from '@/entities/identity-provider/model/types'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useIdentityProviderPresets() {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: queryKeys.identityProviderPresets(realm),
    queryFn: () =>
      apiClient.get<IdentityProviderPreset[]>(`/api/realms/${realm}/identity-providers/presets`),
  })
}
