import { http, HttpResponse } from 'msw'

export const handlers = [
  // Add global handlers here, e.g. for auth check
  http.get('/api/health', () => {
    return HttpResponse.json({ status: 'ok' })
  }),
]
