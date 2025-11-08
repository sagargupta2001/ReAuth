import { Fragment } from 'react'

import { Route, Routes } from 'react-router-dom'

import { usePlugins } from '@/entities/plugin/api/usePlugins'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout.tsx'

import { staticRoutes } from './routerConfig'

export function AppRouter() {
  const { data, isLoading } = usePlugins()

  if (isLoading) return <div>Loading plugins...</div>

  const plugins = data?.manifests || []
  const pluginModules = data?.modules || {}

  return (
    <Routes>
      {/* Render all static routes from the config */}
      {staticRoutes.map(({ path, element: Element, layout: Layout }) => {
        // Use the specified layout, or the DefaultLayout if none is provided
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

      {/* Render all dynamic plugin routes */}
      {plugins.map((plugin) => {
        const Component = pluginModules[plugin.id]
        if (!Component) return null

        const routePath = plugin.frontend.route.startsWith('/')
          ? plugin.frontend.route
          : '/' + plugin.frontend.route

        return (
          <Route
            key={plugin.id}
            path={routePath}
            element={
              <AuthenticatedLayout>
                <Component />{' '}
              </AuthenticatedLayout>
            }
          />
        )
      })}
    </Routes>
  )
}
