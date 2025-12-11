import { useInfiniteQuery } from '@tanstack/react-query'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { apiClient } from '@/shared/api/client'

export interface FlowVersion {
  id: string
  version_number: number
  created_at: string
}

interface VersionListResponse {
  data: FlowVersion[]
  meta: {
    total: number
    page: number
    per_page: number
    last_page: number
  }
}

export function useFlowVersions(flowId: string) {
  const realm = useActiveRealm()

  return useInfiniteQuery({
    queryKey: ['flow-versions', flowId],
    queryFn: async ({ pageParam = 1 }) => {
      // 2. Call API with standard params
      return await apiClient.get<VersionListResponse>(
        `/api/realms/${realm}/flows/${flowId}/versions?page=${pageParam}&per_page=10&sort_by=version_number&sort_dir=desc`,
      )
    },
    initialPageParam: 1,
    getNextPageParam: (lastPage) => {
      // 3. Use meta fields to determine next page
      const { page, last_page } = lastPage.meta
      return page < last_page ? page + 1 : undefined
    },
    select: (data) => ({
      // 4. Flatten the pages for easy rendering
      pages: data.pages.flatMap((page) => page.data),
      pageParams: data.pageParams,
    }),
    enabled: !!flowId,
  })
}
