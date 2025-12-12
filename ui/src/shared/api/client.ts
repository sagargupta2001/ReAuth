import { useSessionStore } from '@/entities/session/model/sessionStore'

// Extend the config to track retries
type RequestConfig = RequestInit & {
  _isRetry?: boolean // Internal flag to prevent infinite loops
}

// Singleton Promise to handle "Thundering Herd" (multiple 401s at once)
let refreshPromise: Promise<string> | null = null

/**
 * Helper to perform the actual refresh call.
 * We do NOT use the main 'request' wrapper here to avoid circular logic.
 */
async function refreshAccessToken(): Promise<string> {
  const response = await fetch('/api/auth/refresh', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include', // CRITICAL: Send the HttpOnly cookie
  })

  if (!response.ok) {
    throw new Error('Session expired')
  }

  const data = await response.json()
  return data.access_token
}

/**
 * The Main Request Wrapper
 */
async function request<T>(endpoint: string, config: RequestConfig = {}): Promise<T> {
  // 1. Get Token from Store
  let token = useSessionStore.getState().accessToken
  const headers = new Headers(config.headers)

  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  if (!headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  // 2. Perform Request
  const response = await fetch(endpoint, {
    ...config,
    headers,
    credentials: 'include',
  })

  // 3. Handle 401 (Unauthorized) - The Interceptor Logic
  if (response.status === 401 && !config._isRetry) {
    // If we are already hitting the refresh endpoint and it fails, stop.
    if (endpoint.includes('/auth/refresh')) {
      useSessionStore.getState().clearSession()
      throw new Error('Session expired')
    }

    try {
      // A. Mutex Logic: If a refresh is already in progress, wait for it.
      if (!refreshPromise) {
        refreshPromise = refreshAccessToken()
      }

      const newToken = await refreshPromise

      // B. Update Global Store
      useSessionStore.getState().setSession(newToken)

      // C. Retry Original Request
      // We pass _isRetry: true to prevent an infinite loop if the server still says 401
      return request<T>(endpoint, { ...config, _isRetry: true })
    } catch (err) {
      // D. Refresh Failed - User is truly logged out
      useSessionStore.getState().clearSession()
      // Optional: Redirect to login via window if not handled by AuthGuard
      // window.location.href = '/#/login'
      throw new Error('Session expired')
    } finally {
      // Reset the promise so future 401s trigger a new refresh
      refreshPromise = null
    }
  }

  // 4. Handle other errors
  if (!response.ok) {
    const errorBody = await response.text()
    let errorMessage = `API Error: ${response.statusText}`
    try {
      const json = JSON.parse(errorBody)
      errorMessage = json.error || errorMessage
    } catch {
      /* ignore json parse error */
    }
    throw new Error(errorMessage)
  }

  if (response.status === 204) {
    return {} as T
  }

  return response.json()
}

// Export shorthand methods
export const apiClient = {
  get: <T>(url: string, config?: RequestConfig) => request<T>(url, { ...config, method: 'GET' }),

  post: <T>(url: string, body: unknown, config?: RequestConfig) =>
    request<T>(url, { ...config, method: 'POST', body: JSON.stringify(body) }),

  put: <T>(url: string, body: unknown, config?: RequestConfig) =>
    request<T>(url, { ...config, method: 'PUT', body: JSON.stringify(body) }),

  delete: <T>(url: string, config?: RequestConfig) =>
    request<T>(url, { ...config, method: 'DELETE' }),
}
