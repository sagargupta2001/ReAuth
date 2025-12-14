import { useMemo, useState } from 'react'

import {
  CheckCircle2,
  Globe,
  LayoutTemplate,
  ListFilter,
  Plus,
  Search,
  ShieldCheck,
  User,
  Zap,
} from 'lucide-react'
import { NavLink } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Separator } from '@/components/separator'
import type { FlowType, UnifiedFlowDto } from '@/entities/flow/model/types'
import { useFlows } from '@/features/flow/api/useFlows.ts'
import { CreateFlowDialog } from '@/features/flow/components/CreateFlowDialog.tsx'
import { useFlowBindings } from '@/features/flow/hooks/useFlowBindings.ts'
import { cn } from '@/lib/utils'

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
      return LayoutTemplate
  }
}

const flowTypeOptions: { value: FlowType | 'all'; label: string; icon?: any }[] = [
  { value: 'all', label: 'All Types' },
  { value: 'browser', label: 'Browser', icon: Globe },
  { value: 'registration', label: 'Registration', icon: User },
  { value: 'direct', label: 'Direct Grant', icon: Zap },
  { value: 'reset', label: 'Reset Creds', icon: ShieldCheck },
]

export function FlowsSidebar() {
  // Destructure realmId (string) for URLs and isFlowActive (logic)
  const { isFlowActive, realmId } = useFlowBindings()
  const { data: flows, isLoading } = useFlows()

  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<FlowType | 'all'>('all')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  // Filter & Group Logic
  const { activeFlows, availableFlows } = useMemo(() => {
    if (!flows) return { activeFlows: [], availableFlows: [] }

    // 1. Filter
    const filtered = flows.filter((f) => {
      const matchesSearch = f.alias.toLowerCase().includes(search.toLowerCase())
      const matchesType = typeFilter === 'all' || f.type === typeFilter
      return matchesSearch && matchesType
    })

    // 2. Group
    const active: UnifiedFlowDto[] = []
    const available: UnifiedFlowDto[] = []

    filtered.forEach((flow) => {
      if (isFlowActive(flow)) {
        active.push(flow)
      } else {
        available.push(flow)
      }
    })

    // 3. Sort (Alphabetical)
    const sortFn = (a: UnifiedFlowDto, b: UnifiedFlowDto) => a.alias.localeCompare(b.alias)

    return {
      activeFlows: active.sort(sortFn),
      availableFlows: available.sort(sortFn),
    }
  }, [flows, search, typeFilter, isFlowActive])

  if (isLoading || !realmId) return null

  return (
    <div className="bg-muted/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      {/* HEADER */}
      <div className="bg-background flex h-14 shrink-0 items-center justify-between border-b px-4">
        <h2 className="text-sm font-semibold tracking-tight">Authentication Flows</h2>
        <Badge variant="secondary" className="text-muted-foreground h-5 px-1.5 text-[10px]">
          {activeFlows.length + availableFlows.length}
        </Badge>
      </div>

      {/* SEARCH & CONTROLS */}
      <div className="space-y-3 p-3">
        <div className="relative">
          <Search className="text-muted-foreground/50 absolute top-2.5 left-2.5 h-4 w-4" />
          <Input
            placeholder="Find a flow..."
            className="bg-background h-9 pl-9 text-sm transition-shadow focus-visible:ring-1"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        <Select value={typeFilter} onValueChange={(val) => setTypeFilter(val as FlowType | 'all')}>
          <SelectTrigger className="bg-background h-8 w-full text-xs">
            <div className="flex items-center gap-2 truncate">
              <ListFilter className="h-3.5 w-3.5 opacity-70" />
              <span className="text-muted-foreground">Type:</span>
              <SelectValue />
            </div>
          </SelectTrigger>
          <SelectContent>
            {flowTypeOptions.map((opt) => {
              const Icon = opt.icon
              return (
                <SelectItem key={opt.value} value={opt.value}>
                  <div className="flex items-center gap-2">
                    {Icon && <Icon className="h-3.5 w-3.5 opacity-70" />}
                    {opt.label}
                  </div>
                </SelectItem>
              )
            })}
          </SelectContent>
        </Select>
      </div>

      <Separator />

      {/* FLOW LIST */}
      <div className="flex-1 overflow-y-auto p-2">
        {/* Active Group */}
        {activeFlows.length > 0 && (
          <div className="mb-4">
            <h3 className="text-muted-foreground/60 mb-2 px-2 text-[10px] font-bold tracking-wider uppercase">
              Active / Bound
            </h3>
            <div className="space-y-0.5">
              {activeFlows.map((flow) => (
                <FlowItem key={flow.id} flow={flow} realmName={realmId} isActiveSection />
              ))}
            </div>
          </div>
        )}

        {/* Separator */}
        {activeFlows.length > 0 && availableFlows.length > 0 && (
          <Separator className="my-2 opacity-50" />
        )}

        {/* Available Group */}
        {availableFlows.length > 0 && (
          <div>
            <h3 className="text-muted-foreground/60 mb-2 px-2 text-[10px] font-bold tracking-wider uppercase">
              Available Flows
            </h3>
            <div className="space-y-0.5">
              {availableFlows.map((flow) => (
                <FlowItem key={flow.id} flow={flow} realmName={realmId} />
              ))}
            </div>
          </div>
        )}

        {/* Empty State */}
        {activeFlows.length === 0 && availableFlows.length === 0 && (
          <div className="flex h-32 flex-col items-center justify-center gap-2 text-center">
            <div className="bg-muted rounded-full p-3">
              <Search className="text-muted-foreground h-4 w-4" />
            </div>
            <p className="text-muted-foreground text-xs">No flows match your filter.</p>
          </div>
        )}
      </div>

      {/* FOOTER */}
      <div className="bg-background mt-auto border-t p-3">
        <Button
          className="w-full justify-start gap-2 shadow-sm"
          size="sm"
          onClick={() => setIsCreateOpen(true)}
        >
          <Plus className="h-4 w-4" />
          Create New Flow
        </Button>
      </div>

      <CreateFlowDialog open={isCreateOpen} onOpenChange={setIsCreateOpen} />
    </div>
  )
}

function FlowItem({
  flow,
  realmName,
  isActiveSection = false,
}: {
  flow: UnifiedFlowDto
  realmName: string
  isActiveSection?: boolean
}) {
  const Icon = getFlowIcon(flow.type)

  return (
    <NavLink
      to={`/${realmName}/flows/${flow.id}`}
      className={({ isActive }) =>
        cn(
          'group flex items-start gap-3 rounded-md px-3 py-2.5 text-sm transition-all',
          'hover:bg-accent/50 hover:text-accent-foreground',
          isActive
            ? 'bg-sidebar-accent text-sidebar-accent-foreground ring-border font-medium shadow-sm ring-1'
            : 'text-muted-foreground',
        )
      }
    >
      <div
        className={cn(
          'bg-background mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-md border shadow-sm transition-colors',
          isActiveSection
            ? 'border-primary/20 text-primary'
            : 'border-border text-muted-foreground/70',
        )}
      >
        <Icon className="h-3.5 w-3.5" />
      </div>

      <div className="flex flex-1 flex-col overflow-hidden">
        <div className="flex items-center justify-between gap-2">
          <span className="text-foreground truncate leading-none">{flow.alias}</span>
          {isActiveSection && <CheckCircle2 className="text-primary/60 h-3 w-3" />}
        </div>
        <span className="text-muted-foreground/60 mt-1.5 truncate text-[10px] capitalize">
          {flow.type} flow
        </span>
      </div>
    </NavLink>
  )
}
