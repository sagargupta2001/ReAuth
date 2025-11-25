import { NavLink } from 'react-router-dom'

import { getRealmPath } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { cn } from '@/lib/utils'

import { sidebarData } from './config/sidebar-data'
import { useSidebarStore } from './model/sidebarStore'

export function SecondarySidebar() {
  const { activeItemId } = useSidebarStore()
  const realm = useActiveRealm()

  const activeItem = sidebarData.navMain.find((i) => i.title === activeItemId)

  // If no item selected, or selected item has no children, render nothing
  if (!activeItem || !activeItem.items) return null

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      <div className="flex h-14 items-center border-b p-4">
        <h2 className="truncate font-semibold">{activeItem.title}</h2>
      </div>

      <div className="flex flex-col gap-1 overflow-y-auto p-2">
        {activeItem.items.map((subItem) => {
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
