import { SETUP_SEALED_STORAGE_KEY } from '@/shared/config/setup'

export const SETUP_COMPLETE_EVENT = 'reauth:setup-complete'

export function markSetupSealed() {
  localStorage.setItem(SETUP_SEALED_STORAGE_KEY, '1')
}

export function clearSetupSeal() {
  localStorage.removeItem(SETUP_SEALED_STORAGE_KEY)
}
