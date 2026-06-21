import type { ElementType, ReactNode } from 'react'

export interface BreadcrumbSibling {
  label: string
  href: string
  icon?: ElementType
}

/**
 * A single resolved node in the header breadcrumb trail.
 */
export interface BreadcrumbNode {
  /** Unique, stable key — the cumulative pathname up to this node. */
  id: string
  label: ReactNode
  /** Present => rendered as a navigable link. Absent + !isCurrent => plain text. */
  href?: string
  icon?: ElementType
  /** The leaf / current page. Rendered non-interactive with aria-current. */
  isCurrent?: boolean
  /** Quick-switch peers shown in a dropdown (absolute hrefs). */
  siblings?: BreadcrumbSibling[]
}

/**
 * A tab within a detail page. The breadcrumb renders the active tab as a
 * quick-switch dropdown so you can change tabs without leaving the pill.
 */
export interface TabDef {
  slug: string
  label: string
  icon?: ElementType
}

/**
 * Declarative description of a known URL path segment. The registry of these is
 * the single source of truth for how the breadcrumb labels routes.
 */
export interface SegmentDef {
  label: string
  icon?: ElementType
  /** Don't render this segment as its own node (structural passthrough). */
  skip?: boolean
  /** Render as plain text even when not the leaf — for sections with no index page. */
  noLink?: boolean
  /** Quick-switch group; urls are realm-relative (begin with '/'). */
  siblings?: { label: string; url: string }[]
}
