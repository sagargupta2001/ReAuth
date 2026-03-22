import { Toaster } from 'sonner'

import { useTheme } from '@/app/providers/ThemeContext'
import { HashRouteNormalizer } from '@/app/HashRouteNormalizer'

import { AppRouter } from './AppRouter'

function App() {
  const { resolvedTheme } = useTheme()

  return (
    <>
      <HashRouteNormalizer />
      <AppRouter />
      <Toaster theme={resolvedTheme} position="top-center" />
    </>
  )
}

export default App
