import { useMutation } from '@tanstack/react-query'

import { oidcApi } from './oidc'

export function useOidcAuth() {
  // Mutation for the initial /authorize call
  const authorizeMutation = useMutation({
    mutationFn: (codeChallenge: string) => oidcApi.authorize(codeChallenge),
  })

  // Mutation for the code -> token exchange
  const exchangeTokenMutation = useMutation({
    mutationFn: ({ code, verifier }: { code: string; verifier: string }) =>
      oidcApi.exchangeToken(code, verifier),
  })

  return {
    authorize: authorizeMutation,
    exchangeToken: exchangeTokenMutation,
  }
}
