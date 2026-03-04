export interface Theme {
  id: string
  realm_id: string
  name: string
  description?: string | null
  is_system: boolean
  created_at: string
  updated_at: string
}

export interface ThemeDetails {
  theme: Theme
  active_version_id?: string | null
  active_version_number?: number | null
}

export interface ThemeVersion {
  id: string
  theme_id: string
  version_number: number
  status: string
  created_at: string
}

export interface ThemeAsset {
  id: string
  theme_id: string
  asset_type: string
  filename: string
  mime_type: string
  byte_size: number
  checksum?: string | null
  created_at: string
  updated_at: string
  url: string
}

export type ThemeNodeType = 'Box' | 'Text' | 'Image' | 'Icon' | 'Input' | 'Component'

export interface ThemeNodeLayout {
  direction?: 'row' | 'column'
  gap?: number
  align?: 'start' | 'center' | 'end' | 'stretch'
  padding?: [number, number, number, number]
}

export interface ThemeNodeSize {
  width?: 'fixed' | 'hug' | 'fill'
  height?: 'fixed' | 'hug' | 'fill'
  width_value?: number | string
  height_value?: number | string
}

export interface ThemeNode {
  id: string
  type: ThemeNodeType
  component?: string
  props?: Record<string, unknown>
  layout?: ThemeNodeLayout
  size?: ThemeNodeSize
  children?: ThemeNode[]
  slots?: Record<string, ThemeNode>
}

export type ThemeBlueprint = ThemeNode[] | { layout?: string; nodes?: ThemeNode[] }

export interface ThemeDraftNode {
  node_key: string
  blueprint: ThemeBlueprint
}

export interface ThemePageTemplate {
  key: string
  label: string
  description: string
  blueprint: ThemeBlueprint
}

export interface ThemeDraft {
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  nodes: ThemeDraftNode[]
}

export interface ThemeSnapshot {
  theme_id: string
  version_id: string
  tokens: Record<string, unknown>
  layout: Record<string, unknown>
  nodes: ThemeNode[]
  assets: ThemeAsset[]
}

export interface ActiveThemeResponse {
  theme: Theme
  active_version_id?: string | null
  active_version_number?: number | null
  pages: ThemePageTemplate[]
}
