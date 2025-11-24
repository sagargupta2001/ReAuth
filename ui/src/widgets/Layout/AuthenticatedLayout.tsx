import type { ReactNode } from 'react'

import { Outlet } from 'react-router-dom'

import { LayoutProvider } from '@/app/providers/layoutProvider'
import { getCookie } from '@/lib/cookies'
import { AppHeader } from '@/widgets/Layout/components/app-header.tsx'
import { AppSidebar } from '@/widgets/Layout/components/app-sidebar'
import { SidebarProvider, useSidebar } from '@/widgets/Sidebar/components/content'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data.ts'
import { useSidebarStore } from '@/widgets/Sidebar/model/sidebarStore.ts'

type AuthenticatedLayoutProps = {
  children?: ReactNode
}

// We extract the inner content to a sub-component so we can use the `useSidebar` hook
function LayoutContent({ children }: { children: React.ReactNode }) {
  const { state } = useSidebar()
  const { activeItemId } = useSidebarStore()

  const activeItem = sidebarData.navMain.find((i) => i.title === activeItemId)
  const showSecondary = !!activeItem?.items

  // Logic must match AppSidebar
  const primaryWidth = state === 'collapsed' ? 'var(--sidebar-width-icon)' : 'var(--sidebar-width)'

  return (
    <div className="bg-background flex min-h-screen w-full pt-14">
      <AppHeader />
      <AppSidebar />

      {/* Dynamic Padding */}
      <div
        className="flex flex-1 flex-col transition-[padding] duration-200 ease-linear"
        style={{
          paddingLeft: showSecondary
            ? `calc(${primaryWidth} + var(--sidebar-width-secondary))`
            : primaryWidth,
        }}
      >
        <main className="flex flex-1 flex-col overflow-x-hidden p-6">{children ?? <Outlet />}</main>
      </div>
    </div>
  )
}

export function AuthenticatedLayout({ children }: AuthenticatedLayoutProps) {
  const defaultOpen = getCookie('sidebar_state') !== 'false'

  return (
    <LayoutProvider>
      <SidebarProvider defaultOpen={defaultOpen}>
        <LayoutContent>{children}</LayoutContent>
      </SidebarProvider>
    </LayoutProvider>
  )
}
