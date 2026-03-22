import { SETUP_SEALED_STORAGE_KEY } from '@/shared/config/setup'

let cachedRequired: boolean | null = null
let inflight: Promise<boolean> | null = null
export const SETUP_COMPLETE_EVENT = 'reauth:setup-complete'

export async function getSetupRequired(): Promise<boolean> {
  if (cachedRequired !== null) {
    return cachedRequired
  }

  const sealedCached = localStorage.getItem(SETUP_SEALED_STORAGE_KEY) === '1'

  if (!inflight) {
    inflight = fetch('/api/system/setup/status', {
      method: 'GET',
      credentials: 'include',
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error('Failed to check setup status.')
        }
        const data = (await response.json()) as { required?: boolean }
        const required = Boolean(data.required)
        cachedRequired = required
        if (required) {
          localStorage.removeItem(SETUP_SEALED_STORAGE_KEY)
        } else {
          localStorage.setItem(SETUP_SEALED_STORAGE_KEY, '1')
        }
        return required
      })
      .catch((err) => {
        if (sealedCached) {
          cachedRequired = false
          return false
        }
        throw err
      })
      .finally(() => {
        inflight = null
      })
  }

  return inflight
}

export function clearSetupStatusCache() {
  cachedRequired = null
  inflight = null
}

export function markSetupSealed() {
  cachedRequired = false
  localStorage.setItem(SETUP_SEALED_STORAGE_KEY, '1')
}
