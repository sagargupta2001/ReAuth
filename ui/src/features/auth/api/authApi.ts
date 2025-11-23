import { apiClient } from '@/shared/api/client'

import type { LoginResponse } from '../model/types'
import type { LoginSchema } from '../schema/loginSchema'

export const authApi = {
  /**
   * Refreshes the access token using the HttpOnly cookie.
   */
  refreshAccessToken: async () => {
    const data = await apiClient.post<{ access_token: string }>('/api/auth/refresh', {})
    return data.access_token
  },

  /**
   * Executes a step in the login flow (Username/Password).
   */
  executeLogin: async (credentials: LoginSchema) => {
    return apiClient.post<LoginResponse>('/api/auth/login/execute', { credentials })
  },
}
