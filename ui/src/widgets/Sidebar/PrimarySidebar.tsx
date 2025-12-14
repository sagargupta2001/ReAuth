import { useLocation } from 'react-router-dom'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/tooltip'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { cn } from '@/lib/utils'
import { SidebarTrigger } from '@/widgets/Sidebar/components'
import { useSidebar } from '@/widgets/Sidebar/components/content'
import type { PrimaryNavItem } from '@/widgets/Sidebar/model/types'

import { sidebarData } from './config/sidebar-data'

// We removed useSidebarStore import. We don't need it here.

interface Props {
  activeItem?: PrimaryNavItem
}

export function PrimarySidebar({ activeItem }: Props) {
  const { state } = useSidebar()
  const navigate = useRealmNavigate()
  const location = useLocation()
  const realm = useActiveRealm()

  // Simplified: Check if this item matches the one passed down from the parent
  const isItemActive = (item: (typeof sidebarData.navMain)[0]) => {
    // 1. If the parent told us this is the active group (e.g. "Settings"), highlight it.
    if (activeItem?.title === item.title) return true

    // 2. Fallback for direct links (like Dashboard) that might not trigger the secondary sidebar
    //    but still need to be highlighted if we are on that route.
    const scopedUrl = `/${realm}${item.url === '/' ? '' : item.url}`

    // Exact match
    if (location.pathname === scopedUrl) return true

    // Prefix match (only if not root)
    return item.url !== '/' && location.pathname.startsWith(scopedUrl)
  }

  const handleItemClick = (item: PrimaryNavItem) => {
    // Logic: Just Navigate.
    // The AppSidebar (Parent) listens to the URL change and handles opening/closing
    // the secondary sidebar automatically. We don't need to set state here.
    if (item.items && item.items.length > 0) {
      // If it's a folder (Settings), go to the first child
      navigate(item.items[0].url)
    } else {
      // If it's a page (Dashboard, Flows), go there
      navigate(item.url)
    }
  }

  return (
    <div
      className={cn(
        'bg-sidebar z-20 flex h-full shrink-0 flex-col items-center gap-2 border-r py-4 transition-all duration-200 ease-linear',
        state === 'collapsed' ? 'w-[var(--sidebar-width-icon)]' : 'w-[var(--sidebar-width)]',
      )}
    >
      {/* Navigation Items */}
      <div className="flex w-full flex-1 flex-col gap-2 px-2">
        {sidebarData.navMain.map((item) => {
          const Icon = item.icon
          const active = isItemActive(item)

          const button = (
            <Button
              variant="ghost"
              className={cn(
                'h-10 justify-start rounded-lg transition-all',
                state === 'collapsed' ? 'w-10 justify-center px-0' : 'w-full px-3',
                active && 'bg-sidebar-accent text-sidebar-accent-foreground',
              )}
              onClick={() => handleItemClick(item)}
            >
              <Icon className="h-5 w-5 shrink-0" />
              {state === 'expanded' && <span className="ml-3 truncate">{item.title}</span>}
            </Button>
          )

          if (state === 'expanded') {
            return <div key={item.title}>{button}</div>
          }

          return (
            <Tooltip key={item.title} delayDuration={0}>
              <TooltipTrigger asChild>{button}</TooltipTrigger>
              <TooltipContent side="right">{item.title}</TooltipContent>
            </Tooltip>
          )
        })}
      </div>

      {/* Trigger Area */}
      <div className="mt-auto flex w-full flex-col items-center gap-2">
        <Separator className="bg-sidebar-border w-full" />
        <SidebarTrigger className={state === 'collapsed' ? 'ml-0' : 'ml-2 self-start'} />
      </div>
    </div>
  )
}
