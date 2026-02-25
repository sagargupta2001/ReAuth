import type { ComponentType, ReactNode } from 'react'

import { Navigate } from 'react-router-dom'

import DashboardPage from '@/pages/DashboardPage'
import { LoginPage } from '@/pages/LoginPage.tsx'
import { LogsPage } from '@/pages/LogsPage.tsx'
import { NotFoundPage } from '@/pages/NotFoundPage'
import { PluginsPage } from '@/pages/PluginsPage.tsx'
import { CachePage } from '@/pages/CachePage.tsx'
import { CreateClientPage } from '@/pages/client/create/CreateClientPage.tsx'
import { EditClientPage } from '@/pages/client/edit/EditClientPage.tsx'
import { ClientsPage } from '@/pages/client/listing/ClientsPage.tsx'
import { FlowDetailsPage } from '@/pages/flow/FlowDetailsPage.tsx'
import { FlowsIndexPage } from '@/pages/flow/FlowsIndexPage.tsx'
import { FlowBuilderPage } from '@/pages/flow/builder/FlowBuilderPage.tsx'
import { CreateRealmPage } from '@/pages/realm/create/CreateRealmPage.tsx'
import { GeneralSettingsPage } from '@/pages/realm/settings/GeneralSettingsPage.tsx'
import { ObservabilitySettingsPage } from '@/pages/realm/settings/ObservabilitySettingsPage.tsx'
import { TokenSettingsPage } from '@/pages/realm/settings/TokenSettingsPage.tsx'
import { CreateRolePage } from '@/pages/roles/create/CreateRolePage.tsx'
import { EditRolePage } from '@/pages/roles/edit/EditRolePage.tsx'
import { RolesPage } from '@/pages/roles/listing/RolesPage.tsx'
import { CreateGroupPage } from '@/pages/groups/create/CreateGroupPage.tsx'
import { EditGroupPage } from '@/pages/groups/edit/EditGroupPage.tsx'
import { GroupsPage } from '@/pages/groups/listing/GroupsPage.tsx'
import EventsDashboard from '@/pages/events/EventsDashboard.tsx'
import TargetDetailsPage from '@/pages/events/TargetDetailsPage.tsx'
import { SessionsPage } from '@/pages/session/listing/SessionsPage.tsx'
import { CreateUserPage } from '@/pages/user/create/CreateUserPage.tsx'
import { EditUserPage } from '@/pages/user/edit/EditUserPage.tsx'
import { UsersPage } from '@/pages/user/listing/UsersPage.tsx'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout.tsx'
import { FlowBuilderLayout } from '@/widgets/Layout/FlowBuilderLayout.tsx'
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
    path: '/:realm/events',
    element: EventsDashboard,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/events/:targetType/:targetId',
    element: TargetDetailsPage,
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
    path: '/:realm/cache',
    element: CachePage,
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
    path: '/:realm/settings/observability',
    element: ObservabilitySettingsPage,
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
    path: '/:realm/clients/:clientId/:tab?',
    element: EditClientPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  { path: '/:realm/users', element: UsersPage, layout: AuthenticatedLayout, isProtected: true },
  {
    path: '/:realm/users/new',
    element: CreateUserPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/users/:userId/:tab?',
    element: EditUserPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/sessions',
    element: SessionsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/flows',
    element: FlowsIndexPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/flows/:flowId',
    element: FlowDetailsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/flows/builder',
    element: FlowBuilderPage,
    layout: FlowBuilderLayout,
    isProtected: true,
  },
  {
    path: '/:realm/flows/:flowId/builder',
    element: FlowBuilderPage,
    layout: FlowBuilderLayout,
    isProtected: true,
  },
  {
    path: '/:realm/roles',
    element: RolesPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/groups',
    element: GroupsPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/roles/:roleId/:tab?',
    element: EditRolePage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/groups/:groupId/:tab?',
    element: EditGroupPage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/roles/new',
    element: CreateRolePage,
    layout: AuthenticatedLayout,
    isProtected: true,
  },
  {
    path: '/:realm/groups/new',
    element: CreateGroupPage,
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
