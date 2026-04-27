import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import {
  getHarborJobDetails,
  isHarborJobActive,
  type HarborJobDetails,
} from '@/features/harbor/api/harborApi'
import { queryKeys } from '@/shared/lib/queryKeys'

export function useHarborJobDetails(jobId: string | null) {
  const realm = useActiveRealm()

  return useQuery<HarborJobDetails>({
    queryKey: queryKeys.harborJobDetails(realm, jobId ?? ''),
    queryFn: async () => {
      if (!realm || !jobId) throw new Error('Missing Harbor job context')
      return getHarborJobDetails({ realm, jobId })
    },
    enabled: !!realm && !!jobId,
    refetchInterval: (query) => {
      const details = query.state.data
      if (!details) return 2_000
      return isHarborJobActive(details.job) ? 2_000 : false
    },
  })
}
