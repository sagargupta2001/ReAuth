import { QueryClient } from '@tanstack/react-query'







export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // We set staleTime to Infinity because our plugin list won't change
      // unless the user restarts the app, or we add a manual refresh.
      staleTime: Infinity,
      refetchOnWindowFocus: false,
    },
  },
})
