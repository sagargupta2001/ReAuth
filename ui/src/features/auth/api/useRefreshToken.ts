import { useMutation } from '@tanstack/react-query'

import { authApi } from './authApi'

export function useRefreshToken() {
  return useMutation({
    // We just point to the API function we already wrote
    mutationFn: authApi.refreshAccessToken,
  })
}
