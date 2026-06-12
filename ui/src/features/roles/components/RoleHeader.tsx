import { Shield } from 'lucide-react'

import { Badge } from '@/components/badge'
import type { Role } from '@/features/roles/api/useRoles.ts'

interface RoleHeaderProps {
  role: Role
}

export function RoleHeader({ role }: RoleHeaderProps) {
  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center px-6 backdrop-blur">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
          <Shield className="text-primary h-5 w-5" />
        </div>

        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h1 className="text-foreground text-lg font-bold tracking-tight">{role.name}</h1>
            {role.client_id ? (
              <Badge variant="outline" className="text-[10px]">
                Client Role
              </Badge>
            ) : (
              <Badge variant="secondary" className="text-[10px]">
                Global Role
              </Badge>
            )}
          </div>
          {role.description ? (
            <span className="text-muted-foreground truncate text-sm">{role.description}</span>
          ) : null}
        </div>
      </div>
    </header>
  )
}
