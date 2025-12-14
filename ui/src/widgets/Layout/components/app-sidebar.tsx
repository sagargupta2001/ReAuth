import { useEffect } from 'react'

import { useLocation } from 'react-router-dom'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { cn } from '@/lib/utils'
import { PrimarySidebar } from '@/widgets/Sidebar/PrimarySidebar'
import { SecondarySidebar } from '@/widgets/Sidebar/SecondarySidebar'
import { useSidebar } from '@/widgets/Sidebar/components/content'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data'
import { useActivePrimaryNavItem } from '@/widgets/Sidebar/hooks/useActivePrimaryNavItem.ts'
import { useSidebarStore } from '@/widgets/Sidebar/model/sidebarStore'

export function AppSidebar() {
  const { state } = useSidebar()
  const { activeItemId, setActiveItem } = useSidebarStore()
  const location = useLocation()
  const realm = useActiveRealm()

  const activeItem = useActivePrimaryNavItem()

  // Only if the active item has a 'segment' (indicating it has sub-content)
  // AND the global sidebar isn't manually collapsed by the user.
  const showSecondary = !!activeItem?.segment && state !== 'collapsed'

  // Calculate Primary Sidebar Width based on state
  // (Assumes CSS vars: --sidebar-width-icon = 3rem, --sidebar-width = 16rem)
  const primaryWidth = state === 'collapsed' ? 'var(--sidebar-width-icon)' : 'var(--sidebar-width)'

  useEffect(() => {
    // 1. Get the current path (e.g. "/master/settings/profile")
    const currentPath = location.pathname

    // 2. Find which Primary Nav Item matches this path
    const matchingItem = sidebarData.navMain.find((item) => {
      // Construct the full path for this item (e.g. "/master/settings")
      const itemPath = `/${realm}${item.url === '/' ? '' : item.url}`

      // Match if current path starts with item path
      // Special case: Don't match root "/" against "/settings"
      if (item.url === '/') {
        return currentPath === itemPath
      }
      return currentPath.startsWith(itemPath)
    })

    // 3. If found and it has children (needs secondary sidebar), set it as active
    if (matchingItem?.items && activeItemId !== matchingItem.title) {
      setActiveItem(matchingItem.title)
    }
    // Optional: If we are on a root link (like Dashboard) and have no children,
    // clear the active item so secondary sidebar closes.
    else if (matchingItem && !matchingItem.items && activeItemId) {
      setActiveItem(null)
    }
  }, [location.pathname, realm, setActiveItem, activeItemId])

  return (
    <div
      className={cn(
        'bg-background fixed left-0 z-40 flex border-r transition-all duration-200 ease-linear',
        'top-14 h-[calc(100vh-3.5rem)]',
        // The container width is the sum of Primary + Secondary (if visible)
        // This relies on calc() to mix CSS vars and dynamic logic
      )}
      style={{
        width: showSecondary
          ? `calc(${primaryWidth} + var(--sidebar-width-secondary))`
          : primaryWidth,
      }}
    >
      {/* Primary is always rendered */}
      <PrimarySidebar activeItem={activeItem} />

      {/* Secondary renders to the right of Primary */}
      <div
        className={cn(
          'overflow-hidden border-l transition-[width,opacity] duration-200 ease-linear',
          showSecondary ? 'w-[var(--sidebar-width-secondary)] opacity-100' : 'w-0 opacity-0',
        )}
      >
        <SecondarySidebar activeItem={activeItem} />
      </div>
    </div>
  )
}
