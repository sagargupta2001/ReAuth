import { useMutation } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client.ts'

type TestRealmEmailPayload = {
  to_address: string
  enabled?: boolean
  from_address?: string | null
  from_name?: string | null
  reply_to_address?: string | null
  smtp_host?: string | null
  smtp_port?: number | null
  smtp_username?: string | null
  smtp_password?: string | null
  smtp_security?: 'starttls' | 'tls' | 'none'
}

export function useTestRealmEmailSettings(realmId: string) {
  return useMutation({
    mutationFn: async (payload: TestRealmEmailPayload) =>
      apiClient.post(`/api/realms/${realmId}/email-settings/test`, payload),
    onSuccess: () => {
      toast.success('Test email sent successfully.')
    },
    onError: (err) => {
      toast.error(`Test email failed: ${err.message}`)
    },
  })
}
