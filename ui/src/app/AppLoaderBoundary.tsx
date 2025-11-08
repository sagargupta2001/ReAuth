import { type FC, type ReactNode, Suspense } from 'react'

import { ErrorBoundary } from 'react-error-boundary'

import { useI18nReady } from '@/shared/lib/hooks/useI18nReady'
import { useStoreHydration } from '@/shared/lib/hooks/useStoreHydration'
import { LocalPageLoader } from '@/shared/ui/local-page-loader.tsx'
import { PageErrorFallback } from '@/shared/ui/page-error-fallback.tsx'
import { GlobalLoader } from '@/widgets/GlobalLoader/GlobalLoader.tsx'

interface Props {
  children: ReactNode
}

export const AppLoaderBoundary: FC<Props> = ({ children }) => {
  const storeHydrated = useStoreHydration()
  const i18nReady = useI18nReady()

  // If the store hasn't hydrated OR i18n isn't ready, show global loader
  if (!storeHydrated || !i18nReady) {
    const message =
      !storeHydrated && !i18nReady
        ? 'Initializing application...'
        : !storeHydrated
          ? 'Restoring session...'
          : 'Loading translations...'

    return <GlobalLoader message={message} />
  }

  // Once both are ready, render with Suspense for code-splitting lazy fallbacks.
  return (
    <ErrorBoundary FallbackComponent={PageErrorFallback}>
      <Suspense fallback={<LocalPageLoader message="Loading application..." />}>
        {children}
      </Suspense>
    </ErrorBoundary>
  )
}
