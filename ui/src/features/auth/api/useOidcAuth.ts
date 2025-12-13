import { useMutation } from '@tanstack/react-query'

import { oidcApi } from './oidc'

export function useOidcAuth() {
  // Mutation for the initial /authorize Navigation
  const authorizeMutation = useMutation({
    mutationFn: async (codeChallenge: string) => {
      const url = oidcApi.getAuthorizeUrl(codeChallenge)
      // Perform Hard Redirect (The App will Unload)
      window.location.href = url
    },
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
