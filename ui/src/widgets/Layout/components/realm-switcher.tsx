import { ChevronsUpDown, GalleryVerticalEnd, Plus } from 'lucide-react'
import { BoxesIcon, Check } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from 'react-router-dom'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Skeleton } from '@/components/skeleton'
import { useRealms } from '@/entities/realm/api/useRealms'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm.ts'
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem } from '@/widgets/Sidebar/components'
import { useSidebar } from '@/widgets/Sidebar/components/content.tsx'

export function RealmSwitcher() {
  const { isMobile } = useSidebar()
  const navigate = useNavigate()
  const currentRealm = useActiveRealm()
  const { data: realms, isLoading } = useRealms()
  const { t } = useTranslation('realm')

  const handleSwitch = (newRealm: string) => navigate(`/${newRealm}`)

  if (isLoading) {
    return (
      <SidebarMenu>
        <SidebarMenuItem>
          <div className="flex h-12 items-center px-2">
            <Skeleton className="h-8 w-8 rounded-lg" />
            <div className="ml-2 space-y-1">
              <Skeleton className="h-3 w-20" />
              <Skeleton className="h-3 w-16" />
            </div>
          </div>
        </SidebarMenuItem>
      </SidebarMenu>
    )
  }

  const activeRealm = realms?.find((r) => r.name === currentRealm) || realms?.[0]

  if (!activeRealm) return null

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <BoxesIcon />
              <div className="grid flex-1 text-start text-sm leading-tight">
                <span className="truncate font-semibold">{activeRealm.name}</span>
              </div>
              <ChevronsUpDown className="ms-auto" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            align="start"
            side={isMobile ? 'bottom' : 'right'}
            sideOffset={4}
          >
            <DropdownMenuLabel className="text-muted-foreground text-xs">
              {t('REALM_SWITCHER.TITLE')}
            </DropdownMenuLabel>
            {realms?.map((realm) => {
              const isSelected = realm.name === currentRealm

              return (
                <DropdownMenuItem
                  key={realm.id}
                  onClick={() => handleSwitch(realm.name)}
                  className="gap-2 p-2"
                >
                  <div className="flex size-6 items-center justify-center rounded-sm border">
                    <GalleryVerticalEnd className="size-4 shrink-0" />
                  </div>

                  {realm.name}

                  <DropdownMenuShortcut>
                    <Check className={isSelected ? 'opacity-100' : 'opacity-0'} />
                  </DropdownMenuShortcut>
                </DropdownMenuItem>
              )
            })}
            <DropdownMenuSeparator />
            <Link to="/create-realm">
              <DropdownMenuItem className="cursor-pointer gap-2 p-2">
                <div className="bg-background flex size-6 items-center justify-center rounded-md border">
                  <Plus className="size-4" />
                </div>
                <div className="text-muted-foreground font-medium">
                  {t('REALM_SWITCHER.CREATE_REALM_BTN')}
                </div>
              </DropdownMenuItem>
            </Link>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
