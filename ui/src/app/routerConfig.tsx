import type { ComponentType, ReactNode } from 'react'

import { Navigate } from 'react-router-dom'

import DashboardPage from '@/pages/DashboardPage'
import { LoginPage } from '@/pages/LoginPage.tsx'
import { LogsPage } from '@/pages/LogsPage.tsx'
import { NotFoundPage } from '@/pages/NotFoundPage'
import { PluginsPage } from '@/pages/PluginsPage.tsx'
import { CreateRealmPage } from '@/pages/realm/create/CreateRealmPage.tsx'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout.tsx'
import { LoginLayout } from '@/widgets/Layout/LoginLayout.tsx'
import { MinimalLayout } from '@/widgets/Layout/MinimalLayout.tsx'

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
    element: () => <Navigate to="/master" replace />,
    isProtected: true,
  },
  {
    path: '/:realm',
    element: DashboardPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/plugins',
    element: PluginsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/logs',
    element: LogsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/create-realm',
    element: CreateRealmPage,
    layout: MinimalLayout,
    isProtected: true,
  },
  {
    path: '/:realm/login',
    element: LoginPage,
    layout: LoginLayout,
    isProtected: false,
  },
  {
    path: '*',
    element: NotFoundPage,
    isProtected: false,
  },
]
