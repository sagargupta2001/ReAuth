import type { FunctionComponent } from 'react'

import type { AuthScreenProps } from '@/entities/auth/model/screenTypes.ts'

import { UsernamePasswordScreen } from '@/features/auth/screens/UsernamePasswordScreen'

// Define the keys the backend sends (e.g., "core.auth.password")
const SCREEN_MAP: Record<string, FunctionComponent<AuthScreenProps>> = {
  // Legacy support
  FORM: UsernamePasswordScreen,

  // New Node IDs
  'core.auth.password': UsernamePasswordScreen,

  // Future examples:
  // 'core.auth.otp': OtpScreen,
  // 'core.consent': ConsentScreen,
}

export const getScreenComponent = (screenId: string): FunctionComponent<AuthScreenProps> | null => {
  // 1. Direct Match
  if (SCREEN_MAP[screenId]) return SCREEN_MAP[screenId]

  // 2. Fuzzy Match (Fallback for dynamic IDs like "password-form-1")
  if (screenId.includes('password') || screenId.includes('login')) return UsernamePasswordScreen

  return null
}
