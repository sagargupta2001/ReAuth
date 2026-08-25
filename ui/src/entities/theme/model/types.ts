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
  /**
   * Cross-axis alignment. Use `baseline` for a row mixing text of different
   * sizes or line heights — `center` lines up boxes, not text.
   */
  align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline'
  /** Main-axis distribution. */
  justify?: 'start' | 'center' | 'end' | 'between' | 'around'
  padding?: [number, number, number, number]
}

export interface ThemeNodeSize {
  width?: 'fixed' | 'hug' | 'fill'
  height?: 'fixed' | 'hug' | 'fill'
  width_value?: number | string
  height_value?: number | string
}

/**
 * Grouped styling a node carries.
 *
 * Replaces the flat `props` bag for anything visual. `props` keeps *content*
 * (`text`, `label`, `href`, `name`, `placeholder`) and *behaviour* (`visible`,
 * `visible_if`, `slot`); `layout` and `size` stay separate because they
 * describe how a node arranges children and how it occupies space, not how it
 * paints.
 *
 * Every group is available on every node type. A group a renderer cannot apply
 * to that type is ignored rather than being an error, so a styling capability
 * is added once instead of once per block.
 */
export interface FillStyle {
  /** Literal colour or a design-token reference. */
  color?: string
}

export interface StrokeStyle {
  color?: string
  width?: number
}

export interface CornerStyle {
  radius?: number | string
}

export interface SpacingStyle {
  /** Space around the block. Distinct from a container's inner padding. */
  padding?: number
  margin_top?: number
  margin_bottom?: number
}

export interface TypographyStyle {
  size?: string
  weight?: string
  color?: string
  align?: 'left' | 'center' | 'right'
}

export interface NodeStyle {
  fill?: FillStyle
  stroke?: StrokeStyle
  corners?: CornerStyle
  spacing?: SpacingStyle
  typography?: TypographyStyle
  /**
   * Styling for the parts a composed component expands into, keyed by part
   * name (an `Input`'s `label` and `field`).
   *
   * Parts are addressable for styling but not for composition — they are
   * render-time nodes and never appear in the authored tree. This is what
   * replaces the nine `label_*` / `field_*` props.
   */
  parts?: Record<string, NodeStyle>
}

export interface ThemeNode {
  id: string
  type: ThemeNodeType
  component?: string
  props?: Record<string, unknown>
  layout?: ThemeNodeLayout
  size?: ThemeNodeSize
  style?: NodeStyle
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
  category: 'auth' | 'consent' | 'awaiting_action' | 'verification' | 'mfa' | 'notification' | 'error' | 'custom'
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
