import type { ReactNode } from 'react'

import { Outlet, useLocation } from 'react-router-dom'

import { LayoutProvider } from '@/app/providers/layoutProvider'
import { getCookie } from '@/lib/cookies'
import {
  UnsavedChangesProvider,
  useUnsavedChanges,
} from '@/shared/context/UnsavedChangesContext.tsx'
import { FloatingActionBar } from '@/shared/ui/floating-action-bar.tsx'
import { AppHeader } from '@/widgets/Layout/components/app-header.tsx'
import { AppSidebar } from '@/widgets/Layout/components/app-sidebar'
import { SidebarProvider, useSidebar } from '@/widgets/Sidebar/components/content'
import { useActivePrimaryNavItem } from '@/widgets/Sidebar/hooks/useActivePrimaryNavItem.ts'

type AuthenticatedLayoutProps = {
  children?: ReactNode
}

function LayoutContent({ children }: { children: ReactNode }) {
  const { state } = useSidebar()
  const { isDirty, isPending, triggerSave, triggerReset } = useUnsavedChanges()
  const location = useLocation()

  const activeItem = useActivePrimaryNavItem()
  const showSecondary =
    !!activeItem && (!!activeItem.items || !!activeItem.segment) && state !== 'collapsed'

  const primaryWidth = state === 'collapsed' ? 'var(--sidebar-width-icon)' : 'var(--sidebar-width)'

  return (
    <div className="bg-background flex min-h-screen w-full pt-14">
      <AppHeader />
      <AppSidebar />
      <div
        className="flex flex-1 flex-col transition-[padding] duration-200 ease-linear"
        style={{
          paddingLeft: showSecondary
            ? `calc(${primaryWidth} + var(--sidebar-width-secondary))`
            : primaryWidth,
        }}
      >
        <main className="flex flex-1 flex-col overflow-x-hidden h-[calc(100vh-64px)]">
          <div key={location.pathname} className="flex h-full flex-1 flex-col">
            {children ?? <Outlet />}
          </div>
        </main>
      </div>
      <FloatingActionBar
        isOpen={isDirty}
        isPending={isPending}
        onSave={triggerSave}
        onReset={triggerReset}
        className="md:right-8 md:left-[calc(var(--sidebar-width)+2rem)]"
      />
    </div>
  )
}

export function AuthenticatedLayout({ children }: AuthenticatedLayoutProps) {
  const defaultOpen = getCookie('sidebar_state') !== 'false'

  return (
    <LayoutProvider>
      <SidebarProvider defaultOpen={defaultOpen}>
        <UnsavedChangesProvider>
          <LayoutContent>{children}</LayoutContent>
        </UnsavedChangesProvider>
      </SidebarProvider>
    </LayoutProvider>
  )
}
