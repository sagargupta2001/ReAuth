import type { ReactNode } from 'react'

import { SetupGate } from '@/app/SetupGate'

type AuthLayoutProps = {
  children: ReactNode
}

export function LoginLayout({ children }: AuthLayoutProps) {
  return (
    <div className="min-h-svh w-full">
      <SetupGate>{children}</SetupGate>
    </div>
  )
}
