import { useMemo, useState } from 'react'

import {
  Copy,
  EditIcon,
  GitBranch,
  Globe,
  Lock,
  MoreHorizontal,
  Plus,
  Search,
  ShieldCheck,
  User,
  Zap,
} from 'lucide-react'
import { NavLink } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Separator } from '@/components/separator'
import type { FlowType, UnifiedFlowDto } from '@/entities/flow/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useFlows } from '@/features/flow/api/useFlows.ts'
import { CreateFlowDialog } from '@/features/flow/components/CreateFlowDialog.tsx'
import { cn } from '@/lib/utils'

// Icon mapping based on Flow Type
const getFlowIcon = (type: string) => {
  switch (type) {
    case 'browser':
      return Globe
    case 'registration':
      return User
    case 'direct':
      return Zap
    case 'reset':
      return ShieldCheck
    default:
      return Globe
  }
}

// Sidebar specific icon for category (System vs Custom)
const CategoryIcon = ({ isSystem }: { isSystem: boolean }) => {
  if (isSystem) return <Lock className="text-muted-foreground/70 h-3.5 w-3.5" />
  return <GitBranch className="text-muted-foreground/70 h-3.5 w-3.5" />
}

const flowTypeOptions: { value: FlowType | 'all'; label: string; icon?: any }[] = [
  { value: 'all', label: 'All Types' },
  { value: 'browser', label: 'Browser Login', icon: Globe },
  { value: 'registration', label: 'Registration', icon: User },
  { value: 'direct', label: 'Direct Grant', icon: Zap },
  { value: 'reset', label: 'Reset Credentials', icon: ShieldCheck },
]

export function FlowsSidebar() {
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<FlowType | 'all'>('all')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const { data: flows, isLoading } = useFlows()

  // 1. Filter Logic
  const filteredFlows = useMemo(() => {
    return (flows || []).filter((f) => {
      const matchesSearch = f.alias.toLowerCase().includes(search.toLowerCase())
      // Check type (safely handle if type string casing differs)
      const matchesType = typeFilter === 'all' || f.type === typeFilter
      return matchesSearch && matchesType
    })
  }, [search, typeFilter, flows])

  // 2. Categorization Logic (System vs Custom)
  const { systemFlows, customFlows } = useMemo(() => {
    const system: UnifiedFlowDto[] = []
    const custom: UnifiedFlowDto[] = []

    filteredFlows.forEach((flow) => {
      if (flow.built_in) {
        system.push(flow)
      } else {
        custom.push(flow)
      }
    })

    return { systemFlows: system, customFlows: custom }
  }, [filteredFlows])

  if (isLoading) return null // Or a skeleton loader

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      {/* HEADER */}
      <div className="flex h-14 shrink-0 items-center justify-between border-b px-4">
        <h2 className="font-semibold">Auth Flows</h2>
        <Badge variant="secondary" className="rounded-full px-2 py-0 text-xs">
          {filteredFlows.length}
        </Badge>
      </div>

      {/* SEARCH & FILTER */}
      <div className="space-y-3 p-3 pb-0">
        <div className="relative">
          <Search className="text-muted-foreground absolute top-2.5 left-2 h-4 w-4" />
          <Input
            placeholder="Search flows..."
            className="bg-background h-9 pl-8"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        <Select value={typeFilter} onValueChange={(val) => setTypeFilter(val as FlowType | 'all')}>
          <SelectTrigger className="bg-background h-8 text-xs">
            <SelectValue placeholder="Filter by type" />
          </SelectTrigger>
          <SelectContent>
            {flowTypeOptions.map((opt) => {
              const Icon = opt.icon
              return (
                <SelectItem key={opt.value} value={opt.value}>
                  <div className="flex items-center gap-2">
                    {Icon && <Icon className="h-4 w-4 opacity-70" />}
                    {opt.label}
                  </div>
                </SelectItem>
              )
            })}
          </SelectContent>
        </Select>
      </div>

      <Separator className="mt-3" />

      {/* FLOW LIST */}
      <div className="flex-1 overflow-y-auto p-2">
        <div className="flex flex-col gap-4">
          {/* SYSTEM FLOWS GROUP */}
          {systemFlows.length > 0 && (
            <div>
              <h3 className="text-muted-foreground px-2 pb-2 text-[10px] font-bold tracking-wider uppercase">
                System Flows
              </h3>
              <div className="flex flex-col gap-1">
                {systemFlows.map((flow) => (
                  <FlowSidebarItem key={flow.id} flow={flow} realm={realm} navigate={navigate} />
                ))}
              </div>
            </div>
          )}

          {/* CUSTOM FLOWS GROUP */}
          {customFlows.length > 0 && (
            <div>
              <h3 className="text-muted-foreground px-2 pb-2 text-[10px] font-bold tracking-wider uppercase">
                Custom Flows
              </h3>
              <div className="flex flex-col gap-1">
                {customFlows.map((flow) => (
                  <FlowSidebarItem key={flow.id} flow={flow} realm={realm} navigate={navigate} />
                ))}
              </div>
            </div>
          )}

          {/* EMPTY STATE */}
          {filteredFlows.length === 0 && (
            <div className="text-muted-foreground py-8 text-center text-xs">No flows found</div>
          )}
        </div>
      </div>

      {/* FOOTER */}
      <div className="bg-background/50 mt-auto border-t p-3 backdrop-blur-sm">
        <Button className="w-full gap-2" onClick={() => setIsCreateOpen(true)}>
          <Plus className="h-4 w-4" /> Create Flow
        </Button>
      </div>
      <CreateFlowDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} />
    </div>
  )
}

// Sub-component for individual list items to keep main component clean
function FlowSidebarItem({
  flow,
  realm,
  navigate,
}: {
  flow: UnifiedFlowDto
  realm: string
  navigate: ReturnType<typeof useRealmNavigate>
}) {
  const TypeIcon = getFlowIcon(flow.type)

  return (
    <div className="group relative flex items-center">
      <NavLink
        to={`/${realm}/flows/${flow.id}`}
        className={({ isActive }) =>
          cn(
            'flex flex-1 flex-col items-start gap-1 rounded-md p-2.5 pr-8 text-sm transition-colors',
            'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
            isActive
              ? 'bg-sidebar-accent text-sidebar-accent-foreground shadow-sm'
              : 'text-muted-foreground',
          )
        }
      >
        <div className="flex w-full items-center gap-2">
          {/* Category Icon (Lock or Branch) */}
          <CategoryIcon isSystem={flow.built_in} />

          <span className="text-foreground truncate font-medium">{flow.alias}</span>
        </div>

        {/* Metadata Row */}
        <div className="mt-1.5 flex items-center gap-2">
          {/* Flow Type Badge */}
          <div className="flex items-center gap-1 text-[10px] opacity-70">
            <TypeIcon className="h-3 w-3" />
            <span className="capitalize">{flow.type}</span>
          </div>

          {/* Draft Indicator (Only show if it has a draft) */}
          {flow.is_draft && (
            <Badge
              variant="outline"
              className="text-primary border-primary/30 ml-auto h-4 px-1 text-[9px] font-normal"
            >
              Draft
            </Badge>
          )}
        </div>
      </NavLink>

      {/* Context Menu */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="absolute top-2 right-1 h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100 data-[state=open]:opacity-100"
          >
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuLabel>Actions</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem>
            <Copy className="mr-2 h-4 w-4" /> Clone Flow
          </DropdownMenuItem>
          {/* Logic for default setting would check realm config, omitted for brevity */}
          {!flow.built_in && (
            <DropdownMenuItem className="text-destructive focus:text-destructive">
              Delete
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={() => navigate(`/flows/${flow.id}/builder`)}>
            <EditIcon className="mr-2 h-4 w-4" />
            {flow.is_draft ? 'Edit Draft' : 'New Draft'}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
