import { Fragment } from 'react'

import { Route, Routes } from 'react-router-dom'

import { AuthGuard } from '@/app/AuthGuard.tsx'
import { usePlugins } from '@/entities/plugin/api/usePlugins'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout.tsx'

// Assuming this is your main layout
import { staticRoutes } from './routerConfig.tsx'

export function AppRouter() {
  const { data } = usePlugins()

  // Get the correct data from the hook
  const pluginStatuses = data?.statuses || []
  const pluginModules = data?.modules || {}

  // Filter for plugins that are 'active' and have a module loaded
  const activePlugins = pluginStatuses.filter(
    (p) => p.status === 'active' && pluginModules[p.manifest.id],
  )

  return (
    <Routes>
      {/* Render all static routes from the config */}
      {staticRoutes.map(({ path, element: Element, layout: Layout, isProtected }) => {
        const LayoutComponent = Layout ?? Fragment
        const page = (
          <LayoutComponent>
            <Element />
          </LayoutComponent>
        )

        return (
          <Route
            key={path}
            path={path}
            element={isProtected ? <AuthGuard>{page}</AuthGuard> : page}
          />
        )
      })}

      {/* Render all DYNAMIC and ACTIVE plugin routes */}
      {activePlugins.map((plugin) => {
        const { manifest } = plugin
        const Component = pluginModules[manifest.id]

        // This check is redundant now but good for safety
        if (!Component) return null

        const routePath = manifest.frontend.route.startsWith('/')
          ? manifest.frontend.route
          : '/' + manifest.frontend.route

        const page = (
          <AuthenticatedLayout>
            <Component />
          </AuthenticatedLayout>
        )

        return (
          <Route
            key={manifest.id}
            path={`/:realm${routePath}`}
            // All plugin routes are protected by default
            element={<AuthGuard>{page}</AuthGuard>}
          />
        )
      })}
    </Routes>
  )
}
