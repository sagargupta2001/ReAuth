import React, { StrictMode, Suspense } from 'react'

import * as ReactDOM from 'react-dom'

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import * as ReactDOMClient from 'react-dom/client'
import { HashRouter as Router } from 'react-router-dom'
import * as jsxRuntime from 'react/jsx-runtime'

import App from '@/app/App.tsx'
import { LayoutProvider } from '@/app/providers/layoutProvider.tsx'
import { ThemeProvider } from '@/app/providers/themeProvider.tsx'
import '@/app/style/index.css'
import { SearchProvider } from '@/features/Search/model/searchContext.tsx'
import '@/shared/config/i18n'
import { DEFAULT_THEME } from '@/shared/config/theme.ts'

// --- Expose React globals before loading any plugin ---
declare global {
  interface Window {
    React: typeof React
    ReactDOM: typeof ReactDOM
    ReactDOMClient: typeof ReactDOMClient
    jsxRuntime: typeof jsxRuntime
  }
}

window.React = React
window.ReactDOM = ReactDOM
window.ReactDOMClient = ReactDOMClient
window.jsxRuntime = jsxRuntime

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // We set staleTime to Infinity because our plugin list won't change
      // unless the user restarts the app, or we add a manual refresh.
      staleTime: Infinity,
      refetchOnWindowFocus: false,
    },
  },
})

const root = ReactDOMClient.createRoot(document.getElementById('root')!)

root.render(
  <StrictMode>
    <Suspense fallback={<div>Loading translations...</div>}>
      <QueryClientProvider client={queryClient}>
        <Router>
          <ThemeProvider defaultTheme={DEFAULT_THEME}>
            <SearchProvider>
              <LayoutProvider>
                <App />
              </LayoutProvider>
            </SearchProvider>
          </ThemeProvider>
        </Router>
      </QueryClientProvider>
    </Suspense>
  </StrictMode>,
)
