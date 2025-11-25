import { useMutation } from '@tanstack/react-query'

import type { LoginSchema } from '@/features/auth/schema/loginSchema.ts'

import { authApi } from './authApi'

export function useLogin() {
  return useMutation({
    mutationFn: (credentials: LoginSchema) => authApi.executeLogin(credentials),
  })
}
