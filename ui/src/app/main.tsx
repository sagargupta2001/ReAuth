import { StrictMode } from 'react'



import { QueryClientProvider } from '@tanstack/react-query';
import * as ReactDOMClient from 'react-dom/client';
import { HashRouter as Router } from 'react-router-dom';



import App from '@/app/App.tsx';
import { AppLoaderBoundary } from '@/app/AppLoaderBoundary.tsx';
import { LayoutProvider } from '@/app/providers/layoutProvider.tsx';
import { ThemeProvider } from '@/app/providers/themeProvider.tsx';
import { queryClient } from '@/app/queryClient.tsx';
import '@/app/style/index.css';
import { SearchProvider } from '@/features/Search/model/searchContext.tsx';
import '@/shared/config/i18n';
import { DEFAULT_THEME } from '@/shared/config/theme.ts';





















const root = ReactDOMClient.createRoot(document.getElementById('root')!)

root.render(
  <StrictMode>
    <ThemeProvider defaultTheme={DEFAULT_THEME}>
      <AppLoaderBoundary>
        <QueryClientProvider client={queryClient}>
          <Router>
            <SearchProvider>
              <LayoutProvider>
                <App />
              </LayoutProvider>
            </SearchProvider>
          </Router>
        </QueryClientProvider>
      </AppLoaderBoundary>
    </ThemeProvider>
  </StrictMode>,
)
