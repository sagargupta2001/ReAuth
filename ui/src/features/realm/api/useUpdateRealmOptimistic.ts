import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { Realm } from '@/entities/realm/model/types'
import { apiClient } from '@/shared/api/client'
import { queryKeys } from '@/shared/lib/queryKeys'

export type UpdateRealmPayload = Partial<Realm>

interface OptimisticContext {
  previousRealm?: Realm
  previousRealms?: Realm[]
}

export function useUpdateRealmOptimistic(realmId: string, realmName: string) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: UpdateRealmPayload) =>
      apiClient.put<Realm>(`/api/realms/${realmId}`, data),
    onMutate: async (data): Promise<OptimisticContext> => {
      await queryClient.cancelQueries({ queryKey: queryKeys.realm(realmName) })
      const previousRealm = queryClient.getQueryData<Realm>(queryKeys.realm(realmName))
      const previousRealms = queryClient.getQueryData<Realm[]>(queryKeys.realms())

      if (previousRealm) {
        queryClient.setQueryData<Realm>(queryKeys.realm(realmName), {
          ...previousRealm,
          ...data,
        })
      }

      if (previousRealms) {
        queryClient.setQueryData<Realm[]>(
          queryKeys.realms(),
          previousRealms.map((realm) =>
            realm.id === realmId ? { ...realm, ...data } : realm,
          ),
        )
      }

      return { previousRealm, previousRealms }
    },
    onError: (error, _data, context) => {
      if (context?.previousRealm) {
        queryClient.setQueryData(queryKeys.realm(realmName), context.previousRealm)
      }
      if (context?.previousRealms) {
        queryClient.setQueryData(queryKeys.realms(), context.previousRealms)
      }
      toast.error(`Update failed: ${error.message}`)
    },
    onSuccess: (updatedRealm) => {
      queryClient.setQueryData<Realm>(queryKeys.realm(realmName), updatedRealm)
      queryClient.setQueryData<Realm[]>(queryKeys.realms(), (old) => {
        if (!old) return [updatedRealm]
        return old.map((realm) => (realm.id === updatedRealm.id ? updatedRealm : realm))
      })
      toast.success('Realm updated')
    },
  })
}
