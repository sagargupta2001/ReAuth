import { useLocation } from 'react-router-dom'

import { Button } from '@/components/button'
import { Separator } from '@/components/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/tooltip'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { cn } from '@/lib/utils'
import { SidebarTrigger } from '@/widgets/Sidebar/components'
import { useSidebar } from '@/widgets/Sidebar/components/content'

import { sidebarData } from './config/sidebar-data'
import { useSidebarStore } from './model/sidebarStore'

export function PrimarySidebar() {
  const { activeItemId, setActiveItem } = useSidebarStore()
  const { state } = useSidebar() // Used for styling based on collapse state
  const navigate = useRealmNavigate()
  const location = useLocation()
  const realm = useActiveRealm()

  const isItemActive = (item: (typeof sidebarData.navMain)[0]) => {
    if (item.items && activeItemId === item.title) return true
    const scopedUrl = `/${realm}${item.url === '/' ? '' : item.url}`
    if (location.pathname === scopedUrl) return true
    return location.pathname.startsWith(scopedUrl) && item.url !== '/'
  }

  const handleItemClick = (item: (typeof sidebarData.navMain)[0]) => {
    if (item.items && item.items.length > 0) {
      // Case A: It has children.
      // 1. Set it as active to show Secondary Sidebar
      setActiveItem(item.title)

      // 2. --- AUTO-NAVIGATE TO FIRST CHILD ---
      const firstChildUrl = item.items[0].url
      navigate(firstChildUrl)
      // ---------------------------------------
    } else {
      // Case B: Direct link.
      setActiveItem(null) // Close secondary sidebar
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

          // 1. Define the Button UI (Shared)
          const button = (
            <Button
              variant="ghost"
              // If expanded, allow button to stretch and show text
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

          // 2. Conditional Rendering:
          // If Expanded: Just show the button (No Tooltip)
          if (state === 'expanded') {
            return <div key={item.title}>{button}</div>
          }

          // If Collapsed: Wrap in Tooltip
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
