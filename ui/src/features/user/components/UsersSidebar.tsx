import { useState } from 'react'

import { Plus } from 'lucide-react'
import { NavLink, useLocation } from 'react-router-dom'

import { Button } from '@/components/button'
import { getRealmPath } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { CreateUserDialog } from '@/features/user/components/CreateUserDialog'
import { cn } from '@/lib/utils'

export function UsersSidebar() {
  const realm = useActiveRealm()
  const { pathname } = useLocation()
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const isInvitations = pathname.includes('/users/invitations')

  const navItems = [
    {
      title: 'All Users',
      url: '/users',
      isActive: pathname.includes('/users') && !isInvitations,
    },
    {
      title: 'Invitations',
      url: '/users/invitations',
      isActive: isInvitations,
    },
  ]

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-(--sidebar-width-secondary) flex-col border-r">
      <div className="bg-background flex h-14 shrink-0 items-center border-b px-4">
        <h2 className="text-lg font-semibold tracking-tight">Users</h2>
      </div>

      <div className="flex flex-1 flex-col gap-1 overflow-y-auto p-2">
        {navItems.map((item) => (
          <NavLink
            key={item.url}
            to={getRealmPath(item.url, realm)}
            className={cn(
              'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
              'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
              item.isActive
                ? 'bg-sidebar-accent text-sidebar-accent-foreground font-medium'
                : 'text-muted-foreground',
            )}
          >
            {item.title}
          </NavLink>
        ))}
      </div>

      <div className="bg-background mt-auto border-t p-3">
        <Button className="w-full gap-2" size="sm" onClick={() => setIsCreateOpen(true)}>
          <Plus className="h-4 w-4" />
          Create User
        </Button>
      </div>

      <CreateUserDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} />
    </div>
  )
}
