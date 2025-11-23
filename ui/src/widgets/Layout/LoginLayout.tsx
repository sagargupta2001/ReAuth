import type { ReactNode } from 'react'

type AuthLayoutProps = {
  children: ReactNode
}

export function LoginLayout({ children }: AuthLayoutProps) {
  return (
    <div className="container grid h-svh max-w-none items-center justify-center">
      <div className="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8">
        <div className="mb-4 flex items-center justify-center">
          <img rel="icon" src="/reauth.svg" alt="logo" className="h-7 w-7" />
          <h1 className="text-xl font-medium">reAuth</h1>
        </div>
        {children}
      </div>
    </div>
  )
}
