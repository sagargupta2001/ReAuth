import { Fragment } from 'react'

import { Route, Routes } from 'react-router-dom'

import { AuthGuard } from '@/app/AuthGuard.tsx'

// Assuming this is your main layout
import { staticRoutes } from './routerConfig.tsx'

export function AppRouter() {
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
    </Routes>
  )
}
