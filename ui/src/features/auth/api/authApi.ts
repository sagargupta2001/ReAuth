import type { AuthExecutionResponse } from '@/features/auth/model/types.ts'
import { apiClient } from '@/shared/api/client'

export const authApi = {
  /**
   * Refreshes the access token using the HttpOnly cookie.
   */
  refreshAccessToken: async (realm: string) => {
    const data = await apiClient.post<{ access_token: string }>(
      `/api/realms/${realm}/auth/refresh`,
      {},
    )
    return data.access_token
  },

  /**
   * Logs out the user by clearing cookies on the server.
   */
  logout: async (realm: string) => {
    return apiClient.post<void>(`/api/realms/${realm}/auth/logout`, {})
  },

  /**
   * 1. START: Initialize the flow.
   * NOTE: The backend now creates the session cookie automatically.
   */
  startFlow: async (realm: string, searchParams?: string) => {
    const query = searchParams || ''
    // Ensure we don't double-add '?' if searchParams already has it
    const safeQuery = query.startsWith('?') ? query : query ? `?${query}` : ''

    return apiClient.get<AuthExecutionResponse>(`/api/realms/${realm}/auth/login${safeQuery}`)
  },

  /**
   * 2. NEXT: Submit data for the current step.
   * The Session ID is now handled automatically via the 'login_session' cookie.
   */
  submitStep: async (realm: string, data: Record<string, any>) => {
    // The backend accepts a generic JSON payload (e.g. { "username": "...", "password": "..." })
    return apiClient.post<AuthExecutionResponse>(`/api/realms/${realm}/auth/login/execute`, data)
  },
}
