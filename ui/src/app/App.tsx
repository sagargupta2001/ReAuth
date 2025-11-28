import { Toaster } from 'sonner'

import { useTheme } from '@/app/providers/themeProvider.tsx'

import { AppRouter } from './AppRouter'

function App() {
  const { resolvedTheme } = useTheme()

  return (
    <>
      <AppRouter />
      <Toaster theme={resolvedTheme} position="top-center" />
    </>
  )
}

export default App
