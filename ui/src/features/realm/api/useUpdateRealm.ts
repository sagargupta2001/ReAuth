import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'

import type { Realm } from '@/entities/realm/model/types.ts'
import { apiClient } from '@/shared/api/client.ts'

// Allow updating any subset of realm fields
type UpdateRealmPayload = Partial<Realm>

export function useUpdateRealm(realmId: string) {
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const params = useParams() // Get current URL params to check for context

  return useMutation({
    mutationFn: (data: UpdateRealmPayload) => {
      return apiClient.put<Realm>(`/api/realms/${realmId}`, data)
    },
    onSuccess: (updatedRealm) => {
      toast.success('Realm settings updated successfully.')

      const oldName = params.realm
      const newName = updatedRealm.name
      const isRenamed = oldName !== newName

      // Update realms list immediately
      queryClient.setQueryData<Realm[]>(['realms'], (old) => {
        if (!old) return [updatedRealm]
        return old.map((r) => (r.id === updatedRealm.id ? updatedRealm : r))
      })

      if (!isRenamed) {
        void queryClient.invalidateQueries({ queryKey: ['realms'] })
        void queryClient.invalidateQueries({ queryKey: ['realm'] })
        return
      }

      // 1) Put the updated realm object under BOTH keys so anyone asking oldName/newName gets valid data
      queryClient.setQueryData(['realm', oldName], updatedRealm)
      queryClient.setQueryData(['realm', newName], updatedRealm)

      // 2) Navigate to the new path

      // Get full hash path: "#/my-realm-12/settings/general"
      const hash = window.location.hash || '#/'

      // Remove "#"
      const path = hash.replace(/^#/, '')

      // Split into segments
      const segments = path.split('/').filter(Boolean)

      // Replace only realm (segment 0)
      segments[0] = updatedRealm.name

      // Construct the new route
      const newHashPath = '/' + segments.join('/')

      navigate(newHashPath, { replace: true })

      // 3) After nav completes, remove the stale old key and invalidate the list
      // setTimeout 0 gives Router a tick to commit the new route
      setTimeout(() => {
        queryClient.removeQueries({ queryKey: ['realm', oldName] })
        void queryClient.invalidateQueries({ queryKey: ['realms'] })
        void queryClient.invalidateQueries({ queryKey: ['realm'] })
      }, 0)
    },
    onError: (err) => {
      toast.error(`Update failed: ${err.message}`)
    },
  })
}
