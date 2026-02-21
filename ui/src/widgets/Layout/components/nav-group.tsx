import { type ReactNode } from 'react'

import { ChevronRight } from 'lucide-react'
import { Link, useLocation } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/collapsible'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import {
  type NavCollapsible,
  type NavGroup as NavGroupProps,
  type NavItem,
  type NavLink,
} from '@/widgets/Layout/model/types'
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/widgets/Sidebar/components'
import { useSidebar } from '@/widgets/Sidebar/components/content.tsx'

export function NavGroup({ title, items }: NavGroupProps) {
  const { state, isMobile } = useSidebar()
  const location = useLocation()
  const currentPath = location.pathname as string
  const realm = useActiveRealm()

  // Helper to prefix URLs
  const resolveUrl = (url: string | undefined) => {
    if (typeof url !== 'string') return ''
    if (url.startsWith('http')) return url
    const cleanUrl = url === '/' ? '' : url
    return `/${realm}${cleanUrl}`
  }

  return (
    <SidebarGroup>
      <SidebarGroupLabel>{title}</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => {
          const key = `${item.title}-${item.url}`
          const resolvedUrl = resolveUrl(item.url as string)

          // --- BRANCH 1: Item has children (Collapsible) ---
          if (item.items && item.items.length > 0) {
            // FIX: Ensure 'url' is undefined for the parent, as per NavCollapsible type
            const collapsibleItem = {
              ...item,
              url: undefined,
              items: item.items.map((sub) => ({
                ...sub,
                url: resolveUrl(sub.url as string),
              })),
            }

            if (state === 'collapsed' && !isMobile) {
              return (
                <SidebarMenuCollapsedDropdown
                  key={key}
                  item={collapsibleItem}
                  currentPath={currentPath}
                />
              )
            }

            return (
              <SidebarMenuCollapsible key={key} item={collapsibleItem} currentPath={currentPath} />
            )
          }

          // --- BRANCH 2: Item is a single link ---
          // We explicitly set items to undefined to satisfy the NavLink type
          const linkItem = {
            ...item,
            url: resolvedUrl,
            items: undefined,
          }

          return <SidebarMenuLink key={key} item={linkItem} currentPath={currentPath} />
        })}
      </SidebarMenu>
    </SidebarGroup>
  )
}

function NavBadge({ children }: { children: ReactNode }) {
  return <Badge className="rounded-full px-1 py-0 text-xs">{children}</Badge>
}

function SidebarMenuLink({ item, currentPath }: { item: NavLink; currentPath: string }) {
  const { setOpenMobile } = useSidebar()
  return (
    <SidebarMenuItem>
      <SidebarMenuButton asChild isActive={checkIsActive(currentPath, item)} tooltip={item.title}>
        <Link to={item.url} onClick={() => setOpenMobile(false)}>
          {item.icon && <item.icon />}
          <span>{item.title}</span>
          {item.badge && <NavBadge>{item.badge}</NavBadge>}
        </Link>
      </SidebarMenuButton>
    </SidebarMenuItem>
  )
}

function SidebarMenuCollapsible({
  item,
  currentPath,
}: {
  item: NavCollapsible
  currentPath: string
}) {
  const { setOpenMobile } = useSidebar()
  return (
    <Collapsible
      asChild
      defaultOpen={checkIsActive(currentPath, item, true)}
      className="group/collapsible"
    >
      <SidebarMenuItem>
        <CollapsibleTrigger asChild>
          <SidebarMenuButton tooltip={item.title}>
            {item.icon && <item.icon />}
            <span>{item.title}</span>
            {item.badge && <NavBadge>{item.badge}</NavBadge>}
            <ChevronRight className="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90 rtl:rotate-180" />
          </SidebarMenuButton>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub>
            {item.items.map((subItem) => (
              <SidebarMenuSubItem key={subItem.title}>
                <SidebarMenuSubButton asChild isActive={checkIsActive(currentPath, subItem)}>
                  <Link to={subItem.url} onClick={() => setOpenMobile(false)}>
                    {subItem.icon && <subItem.icon />}
                    <span>{subItem.title}</span>
                    {subItem.badge && <NavBadge>{subItem.badge}</NavBadge>}
                  </Link>
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
            ))}
          </SidebarMenuSub>
        </CollapsibleContent>
      </SidebarMenuItem>
    </Collapsible>
  )
}

function SidebarMenuCollapsedDropdown({
  item,
  currentPath,
}: {
  item: NavCollapsible
  currentPath: string
}) {
  return (
    <SidebarMenuItem>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <SidebarMenuButton tooltip={item.title} isActive={checkIsActive(currentPath, item)}>
            {item.icon && <item.icon />}
            <span>{item.title}</span>
            {item.badge && <NavBadge>{item.badge}</NavBadge>}
            <ChevronRight className="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
          </SidebarMenuButton>
        </DropdownMenuTrigger>
        <DropdownMenuContent side="right" align="start" sideOffset={4}>
          <DropdownMenuLabel>
            {item.title} {item.badge ? `(${item.badge})` : ''}
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          {item.items.map((sub) => (
            <DropdownMenuItem key={`${sub.title}-${sub.url}`} asChild>
              <Link
                to={sub.url}
                className={`${checkIsActive(currentPath, sub) ? 'bg-secondary' : ''}`}
              >
                {sub.icon && <sub.icon />}
                <span className="max-w-52 text-wrap">{sub.title}</span>
                {sub.badge && <span className="ms-auto text-xs">{sub.badge}</span>}
              </Link>
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuItem>
  )
}

function checkIsActive(currentPath: string, item: NavItem, mainNav = false): boolean {
  const itemUrl = typeof item.url === 'string' ? item.url : item.url?.pathname || ''

  return (
    currentPath === itemUrl ||
    currentPath.split('?')[0] === itemUrl ||
    !!item?.items?.some((i) => {
      const subUrl = typeof i.url === 'string' ? i.url : i.url?.pathname || ''
      return subUrl === currentPath
    }) ||
    (mainNav &&
      currentPath.split('/')[1] !== '' &&
      currentPath.split('/')[1] === itemUrl.split('/')[1])
  )
}
