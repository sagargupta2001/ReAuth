export type FlowType = 'browser' | 'registration' | 'direct' | 'reset'

export interface FlowDraft {
  id: string
  realm_id: string
  name: string
  description?: string
  graph_json: unknown
  flow_type: FlowType
  created_at: string
  updated_at: string
  active_version?: number | null
  built_in: boolean
}

export interface UnifiedFlowDto {
  id: string
  alias: string
  description?: string
  type: FlowType
  built_in: boolean // Corresponds to Rust's built_in
  is_draft: boolean // Corresponds to Rust's is_draft
  status?: string // Optional status field from Rust
}
