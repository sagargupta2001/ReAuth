import { Route } from 'react-router-dom'

import { usePlugins } from '@/entities/plugin/api/usePlugins'
import { AuthenticatedLayout } from '@/widgets/Layout/AuthenticatedLayout'

export function PluginsRoutes() {
  const { data } = usePlugins()

  const plugins = data?.manifests || []
  const pluginModules = data?.modules || {}

  return (
    <>
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
                <Component />
              </AuthenticatedLayout>
            }
          />
        )
      })}
    </>
  )
}
