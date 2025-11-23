import { useLayout } from '@/app/providers/layoutProvider.tsx'
import { Sidebar } from '@/widgets/Sidebar/Sidebar.tsx'
import { SidebarContent, SidebarRail } from '@/widgets/Sidebar/components'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data.ts'

import { NavGroup } from './nav-group'

export function AppSidebar() {
  const { collapsible, variant } = useLayout()
  return (
    <Sidebar collapsible={collapsible} variant={variant}>
      <SidebarContent>
        {sidebarData.navGroups.map((props) => (
          <NavGroup key={props.title} {...props} />
        ))}
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}
