import type { ReactNode } from 'react'

import { RoleSummaryPanel } from '@/features/roles/components/RoleSummaryPanel'
import type { Role } from '@/features/roles/api/useRoles'

interface RoleTabLayoutProps {
  role: Role
  children: ReactNode
}

export function RoleTabLayout({ role, children }: RoleTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <RoleSummaryPanel role={role} />
      </aside>
    </div>
  )
}
