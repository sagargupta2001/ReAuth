import type { ReactNode } from 'react'

import { Outlet } from 'react-router-dom'

import { Slash } from '@/assets/header/slash.tsx'
import { AppHeader } from '@/widgets/Layout/components/app-header.tsx'

type MinimalLayoutProps = {
  children?: ReactNode
}

export function MinimalLayout({ children }: MinimalLayoutProps) {
  return (
    <div className="bg-background flex min-h-screen flex-col">
      <AppHeader
        leftSlot={
          <div className="flex items-center gap-2">
            <img rel="icon" src="/reauth.svg" alt="logo" className="h-7 w-7" />
            <Slash className="inline-block h-5 w-5 shrink-0 leading-none" />
            <span className="text-sm font-semibold">New Realm</span>
          </div>
        }
      />
      <main className="flex flex-1 flex-col">{children ?? <Outlet />}</main>
    </div>
  )
}
