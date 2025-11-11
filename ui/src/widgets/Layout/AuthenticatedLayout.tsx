import type { ReactNode } from 'react'

import { Outlet } from 'react-router-dom'

import { LayoutProvider } from '@/app/providers/layoutProvider.tsx'
import { getCookie } from '@/lib/cookies'
import { cn } from '@/lib/utils'
import { AppSidebar } from '@/widgets/Layout/components/app-sidebar.tsx'
import { SidebarInset } from '@/widgets/Sidebar/components'
import { SidebarProvider } from '@/widgets/Sidebar/components/content.tsx'

type AuthenticatedLayoutProps = {
  children?: ReactNode
}

export function AuthenticatedLayout({ children }: AuthenticatedLayoutProps) {
  const defaultOpen = getCookie('sidebar_state') !== 'false'
  return (
    <LayoutProvider>
      <SidebarProvider defaultOpen={defaultOpen}>
        <AppSidebar />
        <SidebarInset
          className={cn(
            // Set content container, so we can use container queries
            '@container/content',

            // If layout is fixed, set the height
            // to 100svh to prevent overflow
            'has-[[data-layout=fixed]]:h-svh',

            // If layout is fixed and sidebar is inset,
            // set the height to 100svh - spacing (total margins) to prevent overflow
            'peer-data-[variant=inset]:has-[[data-layout=fixed]]:h-[calc(100svh-(var(--spacing)*4))]',
          )}
        >
          {children ?? <Outlet />}
        </SidebarInset>
      </SidebarProvider>
    </LayoutProvider>
  )
}
