import type { ReactNode } from 'react'

type AuthLayoutProps = {
  children: ReactNode
}

export function LoginLayout({ children }: AuthLayoutProps) {
  return (
    <div className="min-h-svh w-full">{children}</div>
  )
}
