import { useQuery } from '@tanstack/react-query'

import { useSessionStore } from '@/entities/session/model/sessionStore'

import type { Realm } from '../model/types'

const fetchRealms = async (token: string | null) => {
  if (!token) return []

  const res = await fetch('/api/realms', {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
  })

  if (!res.ok) throw new Error('Failed to fetch realms')
  return (await res.json()) as Promise<Realm[]>
}

export function useRealms() {
  const { accessToken } = useSessionStore()

  return useQuery({
    queryKey: ['realms'],
    queryFn: () => fetchRealms(accessToken),
    // Only run the query if we are logged in
    enabled: !!accessToken,
  })
}
