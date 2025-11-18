import type { ComponentType, ReactNode } from 'react'

import DashboardPage from '@/pages/DashboardPage'
import { LoginPage } from '@/pages/LoginPage.tsx'
import { LogsPage } from '@/pages/LogsPage.tsx'
import { NotFoundPage } from '@/pages/NotFoundPage'
import { PluginsPage } from '@/pages/PluginsPage.tsx'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout.tsx'

/**
 * Defines the shape of a static route.
 */
export interface RouteConfig {
  path: string
  element: ComponentType
  layout?: ComponentType<{ children: ReactNode }>
  isProtected: boolean
}

/**
 * An array of all static routes in the application.
 * As your app grows, you just add new pages (like a SettingsPage) here.
 */
export const staticRoutes: RouteConfig[] = [
  {
    path: '/',
    element: DashboardPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/plugins',
    element: PluginsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/logs',
    element: LogsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/login',
    element: LoginPage,
    isProtected: false,
  },
  {
    path: '*',
    element: NotFoundPage,
    isProtected: false,
  },
]
