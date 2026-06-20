import { useState } from 'react'

import { Mail, Plus, Users } from 'lucide-react'
import { NavLink } from 'react-router-dom'

import { Button } from '@/components/button'
import { getRealmPath } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { CreateUserDialog } from '@/features/user/components/CreateUserDialog'
import { cn } from '@/lib/utils'

const navItems = [
  { title: 'All Users', url: '/users', end: true, icon: Users },
  { title: 'Invitations', url: '/users/invitations', end: false, icon: Mail },
]

export function UsersSidebar() {
  const realm = useActiveRealm()
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      <div className="flex flex-1 flex-col gap-1 overflow-y-auto p-2">
        {navItems.map((item) => {
          const path = getRealmPath(item.url, realm)
          return (
            <NavLink
              key={item.url}
              to={path}
              end={item.end}
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
              <item.icon className="h-4 w-4" />
              {item.title}
            </NavLink>
          )
        })}
      </div>

      <div className="bg-background mt-auto border-t p-3">
        <Button
          className="w-full justify-start gap-2"
          size="sm"
          onClick={() => setIsCreateOpen(true)}
        >
          <Plus className="h-4 w-4" />
          Create User
        </Button>
      </div>

      <CreateUserDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} />
    </div>
  )
}
