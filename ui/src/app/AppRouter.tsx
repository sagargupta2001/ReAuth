import { Fragment } from 'react'

import { Route, Routes } from 'react-router-dom'

import { usePlugins } from '@/entities/plugin/api/usePlugins'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout'

// Assuming this is your main layout
import { staticRoutes } from './routerConfig'

export function AppRouter() {
  const { data } = usePlugins()

  // 1. Get the correct data from the hook
  const pluginStatuses = data?.statuses || []
  const pluginModules = data?.modules || {}

  // 2. Filter for plugins that are 'active' and have a module loaded
  const activePlugins = pluginStatuses.filter(
    (p) => p.status === 'active' && pluginModules[p.manifest.id],
  )

  return (
    <Routes>
      {/* Render all static routes from the config */}
      {staticRoutes.map(({ path, element: Element, layout: Layout }) => {
        const LayoutComponent = Layout ?? Fragment
        return (
          <Route
            key={path}
            path={path}
            element={
              <LayoutComponent>
                <Element />
              </LayoutComponent>
            }
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

        return (
          <Route
            key={manifest.id}
            path={routePath}
            element={
              <AuthenticatedLayout>
                <Component />
              </AuthenticatedLayout>
            }
          />
        )
      })}
    </Routes>
  )
}
