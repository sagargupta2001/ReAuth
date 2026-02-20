import { useSessionStore } from '@/entities/session/model/sessionStore'

// Extend the config to track retries
type RequestConfig = RequestInit & {
  _isRetry?: boolean // Internal flag to prevent infinite loops
}

// Singleton Promise to handle "Thundering Herd" (multiple 401s at once)
let refreshPromise: Promise<string> | null = null

/**
 * [HELPER] Extract Realm from URL (Hash or Query)
 * Handles:
 * - /#/login?realm=tenant-a
 * - /?realm=tenant-a
 * - Defaults to 'master'
 */
function getRealmFromUrl(): string {
  try {
    // 1. Check Hash Router (e.g., http://localhost:3000/#/page?realm=customer)
    if (window.location.hash.includes('?')) {
      const queryPart = window.location.hash.split('?')[1]
      const params = new URLSearchParams(queryPart)
      if (params.get('realm')) return params.get('realm')!
    }

    // 2. Check Standard Search (e.g., http://localhost:3000/?realm=customer)
    const searchParams = new URLSearchParams(window.location.search)
    if (searchParams.get('realm')) return searchParams.get('realm')!
  } catch (e) {
    // ignore parsing errors
  }

  // 3. Fallback to master
  return 'master'
}

/**
 * Helper to perform the actual refresh call.
 * We do NOT use the main 'request' wrapper here to avoid circular logic.
 */
async function refreshAccessToken(): Promise<string> {
  // [FIX] Dynamically determine the realm
  const realm = getRealmFromUrl()

  // [FIX] Hit the correct Realm-Scoped Endpoint
  const response = await fetch(`/api/realms/${realm}/auth/refresh`, {
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
  const token = useSessionStore.getState().accessToken
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
      console.error('Refresh failed, forcing logout.')
      useSessionStore.getState().clearSession()

      // [OPTIONAL] Redirect to login if needed, but allow AuthGuard to handle it usually
      // const realm = getRealmFromUrl();
      // window.location.href = `/#/login?realm=${realm}`;

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
