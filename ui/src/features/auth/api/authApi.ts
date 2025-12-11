import { apiClient } from '@/shared/api/client'

import type { ExecutionResponse, LoginResponse } from '../model/types'
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

  /**
   * Logs out the user by invalidating the session on the server.
   */
  logout: async () => {
    // We use void because we don't care about the response body, just the cookie clearing
    return apiClient.post<void>('/api/auth/logout', {})
  },

  /**
   * 1. START: Initialize the flow for a specific realm
   */
  startFlow: async (realm: string) => {
    return apiClient.get<ExecutionResponse>(`/api/realms/${realm}/login`)
  },

  /**
   * 2. NEXT: Submit data for the current step
   */
  submitStep: async (sessionId: string, data: Record<string, any>) => {
    return apiClient.post<ExecutionResponse>(`/api/execution/${sessionId}`, data)
  },
}
