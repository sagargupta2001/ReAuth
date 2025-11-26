export interface OidcClient {
  id: string
  realm_id: string
  client_id: string
  client_secret?: string | null
  redirect_uris: string // It comes as a JSON string from the DB
  scopes: string
}

export interface PageMeta {
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface PaginatedResponse<T> {
  data: T[]
  meta: PageMeta
}

// Helper type for the query params
export interface ClientSearchParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}
