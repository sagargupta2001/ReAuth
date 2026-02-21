import { NavLink } from 'react-router-dom'

import { getRealmPath } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { cn } from '@/lib/utils'
import type { PrimaryNavItem } from '@/widgets/Sidebar/model/types'

interface Props {
  item: PrimaryNavItem
}

export function StaticMenuSidebar({ item }: Props) {
  const realm = useActiveRealm()

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      <div className="flex h-14 shrink-0 items-center border-b p-4">
        <h2 className="truncate font-semibold">{item.title}</h2>
      </div>

      <div className="flex flex-col gap-1 overflow-y-auto p-2">
        {item.items?.map((subItem) => {
          const path = getRealmPath(subItem.url, realm)
          return (
            <NavLink
              key={subItem.url}
              to={path}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
                  'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
                  isActive
                    ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
                    : 'text-muted-foreground',
                )
              }
            >
              {subItem.icon && <subItem.icon className="h-4 w-4" />}
              {subItem.title}
            </NavLink>
          )
        })}
      </div>
    </div>
  )
}
