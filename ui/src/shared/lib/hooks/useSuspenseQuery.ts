import { useQuery, type UseQueryOptions, type UseQueryResult } from '@tanstack/react-query'

/**
 * TanStack Query v5 no longer includes `suspense` in the types.
 * This extends UseQueryOptions to include it manually.
 */
type SuspenseQueryOptions<TData, TError> = UseQueryOptions<TData, TError> & {
  suspense?: boolean
}

/**
 * Wrapper around useQuery that enables React Suspense by default.
 */
export function useSuspenseQuery<TData = unknown, TError = Error>(
  options: SuspenseQueryOptions<TData, TError>,
): UseQueryResult<TData, TError> {
  return useQuery({
    ...options,
    // @ts-expect-error - `suspense` is supported at runtime but not typed
    suspense: true,
  })
}
