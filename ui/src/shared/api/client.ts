import { resolveRealmFromLocation } from '@/shared/lib/realm'

// Extend the config to track retries
type RequestConfig = RequestInit & {
  _isRetry?: boolean // Internal flag to prevent infinite loops
  skipContentType?: boolean
}

// Interceptors for Dependency Injection
let getAccessToken: () => string | null = () => null
let setAccessToken: (token: string) => void = () => {}
let clearSession: () => void = () => {}

export function injectAuthInterceptor(
  get: () => string | null,
  set: (token: string) => void,
  clear: () => void
) {
  getAccessToken = get
  setAccessToken = set
  clearSession = clear
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
  return resolveRealmFromLocation('master')
}

/**
 * Helper to perform the actual refresh call.
 * We do NOT use the main 'request' wrapper here to avoid circular logic.
 */
export async function refreshAccessToken(realmOverride?: string): Promise<string> {
  if (!refreshPromise) {
    // [FIX] Dynamically determine the realm unless overridden
    const realm = realmOverride || getRealmFromUrl()

    refreshPromise = fetch(`/api/realms/${realm}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include', // CRITICAL: Send the HttpOnly cookie
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error('Session expired')
        }
        const data = await response.json()
        return data.access_token as string
      })
      .finally(() => {
        refreshPromise = null
      })
  }

  return refreshPromise
}

/**
 * The low-level request wrapper returning a raw Response.
 */
async function requestRaw(endpoint: string, config: RequestConfig = {}): Promise<Response> {
  const token = getAccessToken()
  const headers = new Headers(config.headers)

  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  if (!headers.has('Content-Type') && !config.skipContentType) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(endpoint, {
    ...config,
    headers,
    credentials: 'include',
  })

  if (response.status === 401 && !config._isRetry) {
    if (endpoint.includes('/auth/refresh')) {
      clearSession()
      throw new Error('Session expired')
    }

    try {
      const newToken = await refreshAccessToken()
      setAccessToken(newToken)
      return requestRaw(endpoint, { ...config, _isRetry: true })
    } catch {
      console.error('Refresh failed, forcing logout.')
      clearSession()
      throw new Error('Session expired')
    }
  }

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

  return response
}

/**
 * The JSON request wrapper.
 */
async function request<T>(endpoint: string, config: RequestConfig = {}): Promise<T> {
  const response = await requestRaw(endpoint, config)

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

  postForm: <T>(url: string, body: FormData, config?: RequestConfig) =>
    request<T>(url, { ...config, method: 'POST', body, skipContentType: true }),

  postUrlEncoded: <T>(url: string, body: URLSearchParams, config?: RequestConfig) =>
    request<T>(url, {
      ...config,
      method: 'POST',
      body: body.toString(),
      headers: {
        ...(config?.headers || {}),
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    }),

  raw: (url: string, config?: RequestConfig) => requestRaw(url, config),

  delete: <T>(url: string, config?: RequestConfig) =>
    request<T>(url, { ...config, method: 'DELETE' }),
}
