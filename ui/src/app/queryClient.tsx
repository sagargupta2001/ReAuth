import { QueryClient } from '@tanstack/react-query'







export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Default to aggressive caching; override per-query where freshness matters.
      staleTime: Infinity,
      refetchOnWindowFocus: false,
    },
  },
})
