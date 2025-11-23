import { useSessionStore } from '@/entities/session/model/sessionStore'

type RequestConfig = RequestInit & {
  // You can add custom config here (e.g., skipAuth: true)
}

/**
 * A wrapper around fetch that handles:
 * 1. Base URL (implicit via proxy)
 * 2. Authorization Headers
 * 3. JSON parsing
 * 4. Error handling
 */
async function request<T>(endpoint: string, config: RequestConfig = {}): Promise<T> {
  // Access the token directly from the Zustand store
  // We use .getState() because this is a vanilla JS function, not a React Component
  const token = useSessionStore.getState().accessToken

  const headers = new Headers(config.headers)

  // Automatically inject the token
  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  // Default to JSON
  if (!headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(endpoint, {
    ...config,
    headers,
  })

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

  // Handle empty responses (like 204 No Content)
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
