import type { ReactNode } from 'react'

import { UserProfileSummaryPanel } from './UserProfileSummaryPanel.tsx'

interface UserTabLayoutProps {
  userId: string
  children: ReactNode
}

export function UserTabLayout({ userId, children }: UserTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:h-full xl:grid-cols-[minmax(0,1fr)_20rem] xl:overflow-hidden">
      <div className="min-w-0 xl:h-full xl:overflow-y-auto xl:pr-1">{children}</div>
      <aside className="xl:h-full xl:overflow-hidden">
        <UserProfileSummaryPanel userId={userId} />
      </aside>
    </div>
  )
}
