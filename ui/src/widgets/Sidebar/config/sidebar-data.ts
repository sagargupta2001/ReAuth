import { AppWindow, Group, KeyRound, LayoutDashboard, LucideClockFading, Package, ScrollText, Settings, Users, Workflow } from 'lucide-react';



import type { SidebarData } from '@/widgets/Sidebar/model/types.ts';









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
      title: 'Users',
      url: '/users',
      icon: Users,
    },
    {
      title: 'Roles',
      url: '/roles',
      icon: KeyRound,
    },
    {
      title: 'Groups',
      url: '/groups',
      icon: Group,
    },
    {
      title: 'Sessions',
      url: '/sessions',
      icon: LucideClockFading,
    },
    {
      title: 'Plugins',
      url: '/plugins',
      icon: Package,
    },
    {
      title: 'Observability',
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
