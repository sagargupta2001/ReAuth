import type { ComponentType, ReactNode } from 'react'

import { Navigate } from 'react-router-dom'

import DashboardPage from '@/pages/DashboardPage'
import { LoginPage } from '@/pages/LoginPage.tsx'
import { LogsPage } from '@/pages/LogsPage.tsx'
import { NotFoundPage } from '@/pages/NotFoundPage'
import { PluginsPage } from '@/pages/PluginsPage.tsx'
import { CreateClientPage } from '@/pages/client/create/CreateClientPage.tsx'
import { EditClientPage } from '@/pages/client/edit/EditClientPage.tsx'
import { ClientsPage } from '@/pages/client/listing/ClientsPage.tsx'
import { CreateRealmPage } from '@/pages/realm/create/CreateRealmPage.tsx'
import { GeneralSettingsPage } from '@/pages/realm/settings/GeneralSettingsPage.tsx'
import { TokenSettingsPage } from '@/pages/realm/settings/TokenSettingsPage.tsx'
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
    path: '/:realm/settings/general',
    element: GeneralSettingsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/settings/token',
    element: TokenSettingsPage,
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
    path: '/:realm/clients',
    element: ClientsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/clients/new',
    element: CreateClientPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/clients/:clientId',
    element: EditClientPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/login',
    element: LoginPage,
    layout: LoginLayout,
    isProtected: true,
  },
  {
    path: '*',
    element: NotFoundPage,
    isProtected: false,
  },
]
