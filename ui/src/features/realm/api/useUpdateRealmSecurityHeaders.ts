import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client.ts'

import type { RealmSecurityHeaders } from '@/entities/realm/model/types.ts'

type UpdateRealmSecurityHeadersPayload = {
  x_frame_options?: string | null
  content_security_policy?: string | null
  x_content_type_options?: string | null
  referrer_policy?: string | null
  strict_transport_security?: string | null
}

export function useUpdateRealmSecurityHeaders(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: UpdateRealmSecurityHeadersPayload) =>
      apiClient.put<RealmSecurityHeaders>(`/api/realms/${realmId}/security-headers`, payload),
    onSuccess: (data) => {
      toast.success('Security headers updated successfully.')
      queryClient.setQueryData(['realm-security-headers', realmId], data)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
