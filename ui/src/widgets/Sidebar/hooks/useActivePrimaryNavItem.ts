import { useLocation } from 'react-router-dom'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data.ts'

export function useActivePrimaryNavItem() {
  const location = useLocation()
  const realm = useActiveRealm()

  return sidebarData.navMain.find((item) => {
    const itemPath = `/${realm}${item.url === '/' ? '' : item.url}`

    // Exact match (e.g. /master/flows)
    if (location.pathname === itemPath) return true

    // Nested match (e.g. /master/flows/create starts with /master/flows)
    // Ensure we don't match root "/" against "/settings"
    return item.url !== '/' && location.pathname.startsWith(itemPath)
  })
}
