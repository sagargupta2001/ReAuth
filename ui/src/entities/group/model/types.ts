export interface Group {
  id: string
  parent_id?: string | null
  name: string
  description?: string | null
  sort_order?: number
}
