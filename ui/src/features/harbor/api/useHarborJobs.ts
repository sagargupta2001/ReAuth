import { useQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { isHarborJobActive, listHarborJobs } from '@/features/harbor/api/harborApi'

export function useHarborJobs(limit = 20) {
  const realm = useActiveRealm()

  return useQuery({
    queryKey: ['harbor-jobs', realm, limit],
    queryFn: async () => {
      if (!realm) throw new Error('Missing realm')
      return listHarborJobs({ realm, limit })
    },
    enabled: !!realm,
    refetchInterval: (query) => {
      const jobs = query.state.data
      if (!jobs || jobs.length === 0) return 10_000
      return jobs.some((job) => isHarborJobActive(job)) ? 2_000 : 10_000
    },
  })
}
