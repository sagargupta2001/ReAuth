import { FlowsSidebar } from '@/features/flow/components/FlowSidebar.tsx'
import { GroupsSidebar } from '@/features/group/components/GroupsSidebar'
import { ThemesSidebar } from '@/features/theme/components/ThemesSidebar.tsx'
import { UsersSidebar } from '@/features/user/components/UsersSidebar'
import { StaticMenuSidebar } from '@/widgets/Sidebar/components/StaticMenuSidebar.tsx'

import type { PrimaryNavItem } from './model/types'

interface SecondarySidebarProps {
  activeItem?: PrimaryNavItem
}

export function SecondarySidebar({ activeItem }: SecondarySidebarProps) {
  if (!activeItem) return null

  // --- THE REGISTRY PATTERN ---
  switch (activeItem.segment) {
    case 'groups':
      return <GroupsSidebar />
    case 'flows':
      return <FlowsSidebar />
    case 'themes':
      return <ThemesSidebar />
    case 'users':
      return <UsersSidebar />

    // Default: If it has static items (like Settings), render the generic list
    default:
      if (activeItem.items && activeItem.items.length > 0) {
        return <StaticMenuSidebar item={activeItem} />
      }
      return null
  }
}
