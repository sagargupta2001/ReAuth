export type FlowType = 'browser' | 'registration' | 'direct' | 'reset'

export interface Flow {
  id: string
  alias: string
  description: string
  type: FlowType
  builtIn: boolean
  isDraft: boolean
  isDefault?: boolean // Can be computed on client or added to DTO later
}

export interface FlowDraft {
  id: string
  realm_id: string
  name: string
  description?: string
  // This stores the raw React Flow JSON ({ nodes: [], edges: [], viewport: ... })
  graph_json: any
  created_at: string
  updated_at: string
}

export interface NodeMetadata {
  id: string
  category: string
  display_name: string
  description: string
  icon: string
  inputs: string[]
  outputs: string[]
  // JSON Schema for the configuration form
  config_schema: any
}

export interface FlowVersion {
  id: string
  draft_id: string
  version_number: number
  execution_artifact: any
  checksum: string
  created_at: string
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
