import { Fragment } from 'react'

import { Route, Routes } from 'react-router-dom'

import { PluginsRoutes } from '@/app/PluginRoutes.tsx'
import { staticRoutes } from '@/app/routerConfig.tsx'

export function AppRouter() {
  return (
    <Routes>
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

      {/* Plugin routes now handled separately */}
      <Route path="/*" element={<PluginsRoutes />} />
    </Routes>
  )
}
