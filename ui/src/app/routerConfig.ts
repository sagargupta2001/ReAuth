import type { ComponentType, ReactNode } from 'react'

import DashboardPage from '@/pages/DashboardPage'
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
  },
  {
    path: '/plugins',
    element: PluginsPage,
    layout: AuthenticatedLayout,
  },
  {
    path: '/logs',
    element: LogsPage,
    layout: AuthenticatedLayout,
  },
  {
    path: '*',
    element: NotFoundPage,
  },
]
