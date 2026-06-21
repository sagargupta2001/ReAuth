import {
  Anchor,
  AppWindow,
  Fingerprint,
  Group,
  KeyRound,
  LayoutDashboard,
  LucideClockFading,
  Palette,
  Settings,
  Users,
  Webhook,
  Workflow,
  Activity
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
      title: 'Users',
      url: '/users',
      icon: Users,
      segment: 'users',
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
      segment: 'groups',
      secondaryWidth: '20rem',
    },
    {
      title: 'Sessions',
      url: '/sessions',
      icon: LucideClockFading,
    },
    {
      title: 'Webhooks',
      url: '/events',
      icon: Webhook,
    },
    {
      title: 'Logs',
      url: '/logs',
      icon: Activity,
    },
    {
      title: 'Harbor',
      url: '/harbor',
      icon: Anchor,
    },
    {
      title: 'Clients',
      url: '/clients',
      icon: AppWindow,
    },
    {
      title: 'Identity Providers',
      url: '/identity-providers',
      icon: Fingerprint,
    },
    {
      title: 'Flows', // This Key will trigger the custom sidebar
      url: '/flows',
      icon: Workflow,
      segment: 'flows',
    },
    {
      title: 'Theme',
      url: '/themes',
      icon: Palette,
      segment: 'themes',
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
          title: 'Email',
          url: '/settings/email',
        },
        {
          title: 'Security Defenses',
          url: '/settings/security',
        },
        {
          title: 'Identity Brokering',
          url: '/settings/identity-brokering',
        },
        {
          title: 'Recovery',
          url: '/settings/recovery',
        },
        {
          title: 'Token',
          url: '/settings/token',
        },
        {
          title: 'Observability',
          url: '/settings/observability',
        },
      ],
    },
  ],
}
