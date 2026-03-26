import { useMutation, useQueryClient } from '@tanstack/react-query'

import { apiClient } from '@/shared/api/client'
import { SETUP_COMPLETE_EVENT, markSetupSealed } from '@/shared/lib/setupStatus'
import { queryKeys } from '@/shared/lib/queryKeys'

export type SetupBootstrapPayload = {
  token: string
  username: string
  password: string
}

export function useSetupBootstrap() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (payload: SetupBootstrapPayload) =>
      apiClient.post('/api/system/setup', payload),
    onSuccess: () => {
      markSetupSealed()
      queryClient.setQueryData(queryKeys.setupStatus(), { required: false })
      window.dispatchEvent(new Event(SETUP_COMPLETE_EVENT))
    },
  })
}
