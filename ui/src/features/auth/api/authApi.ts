export const refreshAccessToken = async () => {
  // The browser automatically sends the HttpOnly cookie
  const res = await fetch('/api/auth/refresh', {
    method: 'POST',
  })

  if (!res.ok) {
    throw new Error('Failed to refresh token')
  }

  const data = await res.json()
  return data.access_token
}
