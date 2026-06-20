import type { ReactNode } from 'react'

import { UserProfileSummaryPanel } from './UserProfileSummaryPanel.tsx'

interface UserTabLayoutProps {
  userId: string
  children: ReactNode
}

export function UserTabLayout({ userId, children }: UserTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <UserProfileSummaryPanel userId={userId} />
      </aside>
    </div>
  )
}
