import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import {
  getHarborJobDetails,
  isHarborJobActive,
  type HarborJobDetails,
} from '@/features/harbor/api/harborApi'

export function useHarborJobDetails(jobId: string | null) {
  const realm = useActiveRealm()

  return useQuery<HarborJobDetails>({
    queryKey: ['harbor-job-details', realm, jobId],
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
