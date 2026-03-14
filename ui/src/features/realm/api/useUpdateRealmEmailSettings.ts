import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { apiClient } from '@/shared/api/client.ts'

import type { RealmEmailSettings } from '@/entities/realm/model/types.ts'

type UpdateRealmEmailSettingsPayload = {
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

export function useUpdateRealmEmailSettings(realmId: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (payload: UpdateRealmEmailSettingsPayload) =>
      apiClient.put<RealmEmailSettings>(`/api/realms/${realmId}/email-settings`, payload),
    onSuccess: (data) => {
      toast.success('Email settings updated successfully.')
      queryClient.setQueryData(['realm-email-settings', realmId], data)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
