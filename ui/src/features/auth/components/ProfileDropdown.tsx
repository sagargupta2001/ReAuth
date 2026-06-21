import { useCurrentUser } from '@/features/auth/api/useCurrentUser'
import { useLogout } from '@/features/auth/api/useLogout.ts'
import { Avatar, AvatarFallback } from '@/shared/ui/avatar.tsx'
import { Button } from '@/shared/ui/button.tsx'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/shared/ui/dropdown-menu.tsx'
import type { User } from '@/entities/user/model/types'

function getInitials(user: Pick<User, 'username' | 'first_name' | 'last_name'>): string {
  if (user.first_name && user.last_name) {
    return `${user.first_name[0]}${user.last_name[0]}`.toUpperCase()
  }
  if (user.first_name) return user.first_name.slice(0, 2).toUpperCase()
  return user.username.slice(0, 2).toUpperCase()
}

export function ProfileDropdown() {
  const logoutMutation = useLogout()
  const { data: currentUser } = useCurrentUser()

  return (
    <DropdownMenu modal={false}>
      <DropdownMenuTrigger asChild>
        <Button variant="secondary" className="relative h-8 w-8 rounded-full">
          <Avatar className="h-8 w-8">
            <AvatarFallback className="text-xs">
              {currentUser ? getInitials(currentUser) : '…'}
            </AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56" align="end" forceMount>
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col gap-1.5">
            <p className="text-sm leading-none font-medium">
              {currentUser?.username ?? '—'}
            </p>
            <p className="text-muted-foreground text-xs leading-none">
              {currentUser?.email ?? '—'}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" onClick={() => logoutMutation.mutate()}>
          Sign out
          <DropdownMenuShortcut className="text-current">⇧⌘Q</DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
