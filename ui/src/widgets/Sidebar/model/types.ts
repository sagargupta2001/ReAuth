import type { ElementType } from 'react'

export interface NavItem {
  title: string
  url: string
  icon?: ElementType
  badge?: string
}

export interface PrimaryNavItem {
  title: string
  url: string // Base URL for highlighting (e.g. "/settings")
  icon: ElementType // Mandatory for primary rail
  items?: NavItem[] // If present, triggers Secondary Sidebar
  // This matches the URL path segment (e.g. "flows", "settings")
  // If present, it implies this item owns a Secondary Sidebar view.
  segment?: string
}

export interface SidebarData {
  user: {
    name: string
    email: string
    avatar: string
  }
  navMain: PrimaryNavItem[]
}
