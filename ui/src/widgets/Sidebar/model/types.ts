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
}

export interface SidebarData {
  user: {
    name: string
    email: string
    avatar: string
  }
  navMain: PrimaryNavItem[] // Renamed from navGroups
}
