import {
  AppWindow,
  KeySquare,
  LayoutDashboard,
  Package,
  ScrollText,
  Settings,
  Workflow,
} from 'lucide-react'

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
      title: 'Overview',
      url: '/',
      icon: LayoutDashboard,
    },
    {
      title: 'Access Control',
      url: '/access',
      icon: KeySquare,
      segment: 'access',
      items: [
        {
          title: 'Users',
          url: '/access/users',
        },
        {
          title: 'Sessions',
          url: '/access/sessions',
        },
        {
          title: 'Roles',
          url: '/access/roles',
        },
        {
          title: 'Groups',
          url: '/access/groups',
        },
      ],
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
      title: 'Flows', // This Key will trigger the custom sidebar
      url: '/flows',
      icon: Workflow,
      segment: 'flows',
    },
    {
      title: 'Settings',
      url: '/settings',
      icon: Settings,
      segment: 'settings', // Maps to /:realm/settings
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
