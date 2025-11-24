import { LayoutDashboard, Package, ScrollText, Settings, UserCog, Wrench } from 'lucide-react'

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
      title: 'Realm Settings',
      url: '/settings',
      icon: Settings,
      // These children will appear in Secondary Sidebar
      items: [
        {
          title: 'General',
          url: '/settings/general',
          icon: UserCog,
        },
        {
          title: 'Token',
          url: '/settings/token',
          icon: Wrench,
        },
      ],
    },
  ],
}
