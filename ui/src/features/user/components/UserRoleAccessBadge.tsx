import { Badge } from '@/components/badge'
import type { UserRoleRow } from '@/features/user/api/useUserRoles'

interface UserRoleAccessBadgeProps {
  role: UserRoleRow
}

export function UserRoleAccessBadge({ role }: UserRoleAccessBadgeProps) {
  const accessType = role.is_direct ? 'direct' : role.is_effective ? 'effective' : 'none'

  switch (accessType) {
    case 'direct':
      return (
        <Badge variant="secondary" className="w-20 justify-center">
          Direct
        </Badge>
      )
    case 'effective':
      return (
        <Badge variant="muted" className="min-w-20 justify-center">
          Effective
        </Badge>
      )
    case 'none':
      return (
        <span className="text-muted-foreground inline-block w-20 text-center text-xs">-</span>
      )
  }
}
