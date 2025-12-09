import type { ReactNode } from 'react'

import { Outlet } from 'react-router-dom'

export function FlowBuilderLayout({ children }: { children: ReactNode }) {
  return (
    <div className="bg-background text-foreground flex h-screen w-screen flex-col overflow-hidden">
      <main className="flex flex-1 overflow-hidden">{children ?? <Outlet />}</main>
    </div>
  )
}
