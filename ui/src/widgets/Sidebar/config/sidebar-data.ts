import { LucideHome, LucideLogs, Package, Settings, UserCog, Wrench } from 'lucide-react'

import type { SidebarData } from '@/widgets/Sidebar/model/types.ts'

export const sidebarData: SidebarData = {
  user: {
    name: 'sagar',
    email: 'sagar@gmail.com',
    avatar: '/avatars/shadcn.jpg',
  },
  navGroups: [
    {
      title: 'General',
      items: [
        {
          title: 'Realm Overview',
          url: '/',
          icon: LucideHome,
        },
        {
          title: 'Plugins',
          url: '/plugins',
          icon: Package,
        },
        {
          title: 'Logs',
          url: '/logs',
          icon: LucideLogs,
        },
      ],
    },
    {
      title: 'Other',
      items: [
        {
          title: 'Realm Settings',
          icon: Settings,
          items: [
            {
              title: 'Profile',
              url: '/settings',
              icon: UserCog,
            },
            {
              title: 'Account',
              url: '/settings/account',
              icon: Wrench,
            },
          ],
        },
      ],
    },
  ],
}
