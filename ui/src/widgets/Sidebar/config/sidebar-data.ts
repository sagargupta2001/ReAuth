import { AppWindow, LayoutDashboard, Package, ScrollText, Settings } from 'lucide-react'

import type { SidebarData } from '@/widgets/Sidebar/model/types.ts'

export const sidebarData: SidebarData = {
  user: {
    name: 'Admin User',
    email: 'admin@reauth.io',
    avatar: '/avatars/default.jpg',
  },
  // Top-level items for the Primary Sidebar
  navMain: [
    {
      title: 'Realm Overview',
      url: '/',
      icon: LayoutDashboard,
    },
    {
      title: 'Plugins',
      url: '/plugins',
      icon: Package,
    },
    {
      title: 'Logs',
      url: '/logs',
      icon: ScrollText,
    },
    {
      title: 'Clients',
      url: '/clients',
      icon: AppWindow,
    },
    {
      title: 'Realm Settings',
      url: '/settings',
      icon: Settings,
      // These children will appear in Secondary Sidebar
      items: [
        {
          title: 'General',
          url: '/settings/general',
        },
        {
          title: 'Token',
          url: '/settings/token',
        },
      ],
    },
  ],
}
