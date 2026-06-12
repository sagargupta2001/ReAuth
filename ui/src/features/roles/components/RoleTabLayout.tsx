import type { ReactNode } from 'react'

import { RoleSummaryPanel } from '@/features/roles/components/RoleSummaryPanel'
import type { Role } from '@/features/roles/api/useRoles'

interface RoleTabLayoutProps {
  role: Role
  children: ReactNode
}

export function RoleTabLayout({ role, children }: RoleTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:h-full xl:grid-cols-[minmax(0,1fr)_20rem] xl:overflow-hidden">
      <div className="min-w-0 xl:h-full xl:overflow-y-auto xl:pr-1">{children}</div>
      <aside className="xl:h-full xl:overflow-hidden">
        <RoleSummaryPanel role={role} />
      </aside>
    </div>
  )
}
