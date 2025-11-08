import { useEffect, useState } from 'react'

import i18n from '@/shared/config/i18n'

export const useI18nReady = () => {
  const [ready, setReady] = useState<boolean>(i18n.isInitialized)

  useEffect(() => {
    if (i18n.isInitialized) {
      setReady(true)
      return
    }

    const handle = () => setReady(true)
    i18n.on('initialized', handle)
    // also catch failed initializations gracefully
    i18n.on('failedLoading', handle)

    return () => {
      i18n.off('initialized', handle)
      i18n.off('failedLoading', handle)
    }
  }, [])

  return ready
}
