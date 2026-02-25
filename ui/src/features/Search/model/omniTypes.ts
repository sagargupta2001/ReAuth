import type { ElementType } from 'react'

export type OmniGroup =
  | 'Suggested Actions'
  | 'Navigation'
  | 'Settings'
  | 'Observability'
  | 'Danger Zone'
  | 'Users'
  | 'Clients'
  | 'Roles'
  | 'Groups'
  | 'Flows'
  | 'Webhooks'

export type OmniStaticItemKind = 'link' | 'action' | 'setting' | 'toggle'

export interface OmniStaticItem {
  id: string
  label: string
  group: OmniGroup
  kind: OmniStaticItemKind
  icon: ElementType
  href?: string
  hash?: string
  description?: string
  keywords?: string[]
  actionId?: string
  toggleId?: string
  suggested?: boolean
}

export interface OmniSearchUser {
  id: string
  username: string
}

export interface OmniSearchClient {
  id: string
  client_id: string
}

export interface OmniSearchRole {
  id: string
  name: string
  description?: string | null
  client_id?: string | null
}

export interface OmniSearchGroup {
  id: string
  name: string
  description?: string | null
}

export interface OmniSearchFlow {
  id: string
  alias: string
  description?: string | null
  flow_type: string
  built_in: boolean
  is_draft: boolean
}

export interface OmniSearchWebhook {
  id: string
  name: string
  url: string
  http_method: string
  status: string
}

export interface OmniSearchResponse {
  users: OmniSearchUser[]
  clients: OmniSearchClient[]
  roles: OmniSearchRole[]
  groups: OmniSearchGroup[]
  flows: OmniSearchFlow[]
  webhooks: OmniSearchWebhook[]
}

export type OmniInspectorItem =
  | {
      kind: 'user'
      id: string
      label: string
      subtitle?: string
      href?: string
    }
  | {
      kind: 'client'
      id: string
      label: string
      subtitle?: string
      href?: string
    }
  | {
      kind: 'role'
      id: string
      label: string
      subtitle?: string
      href?: string
    }
  | {
      kind: 'group'
      id: string
      label: string
      subtitle?: string
      href?: string
    }
  | {
      kind: 'flow'
      id: string
      label: string
      subtitle?: string
      description?: string
      href?: string
    }
  | {
      kind: 'webhook'
      id: string
      label: string
      subtitle?: string
      description?: string
      status?: string
      href?: string
    }
  | {
      kind: 'setting'
      id: string
      label: string
      description?: string
      breadcrumb?: string
      href?: string
    }
  | {
      kind: 'action'
      id: string
      label: string
      description?: string
      href?: string
    }
