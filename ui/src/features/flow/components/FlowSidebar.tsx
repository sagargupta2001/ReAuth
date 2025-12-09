import { useMemo, useState } from 'react'

import {
  Check,
  CircleCheck,
  Copy,
  Globe,
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
import type { FlowType } from '@/entities/flow/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useFlows } from '@/features/flow/api/useFlows.ts'
import { cn } from '@/lib/utils'

const getFlowIcon = (type: FlowType) => {
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

  const { data: flows, isLoading } = useFlows()

  const filteredFlows = useMemo(() => {
    // Use real data or empty array
    return (flows || []).filter((f) => {
      const matchesSearch = f.alias.toLowerCase().includes(search.toLowerCase())
      const matchesType = typeFilter === 'all' || f.type === typeFilter
      return matchesSearch && matchesType
    })
  }, [search, typeFilter, flows])

  if (isLoading) return

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      <div className="flex h-14 shrink-0 items-center justify-between border-b px-4">
        <h2 className="font-semibold">Auth Flows</h2>
        <Badge variant="secondary" className="rounded-full px-2 py-0 text-xs">
          {filteredFlows.length}
        </Badge>
      </div>

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

      <div className="flex-1 overflow-y-auto p-2">
        <div className="flex flex-col gap-1">
          {filteredFlows.map((flow) => {
            const Icon = getFlowIcon(flow.type)

            return (
              <div key={flow.id} className="group relative flex items-center">
                <NavLink
                  to={`/${realm}/flows/${flow.id}`}
                  className={({ isActive }) =>
                    cn(
                      'flex flex-1 flex-col items-start gap-1 rounded-md p-3 pr-8 text-sm transition-colors',
                      'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
                      isActive
                        ? 'bg-sidebar-accent text-sidebar-accent-foreground shadow-sm'
                        : 'text-muted-foreground',
                    )
                  }
                >
                  <div className="flex w-full items-center gap-2">
                    <Icon className="h-3.5 w-3.5 opacity-70" />
                    <span className="text-foreground truncate font-medium">{flow.alias}</span>
                    {flow.isDefault && <CircleCheck size={15} color="green" />}
                  </div>

                  <div className="mt-1 flex items-center gap-2 text-xs opacity-70">
                    {flow.builtIn && <Badge variant="muted">Built in</Badge>}
                  </div>
                </NavLink>

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="secondary"
                      size="icon"
                      className="absolute top-1/2 right-1 h-8 w-8 -translate-y-1/2 opacity-0 group-hover:opacity-100 data-[state=open]:opacity-100"
                    >
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuLabel>Flow Actions</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem>
                      <Copy className="mr-2 h-4 w-4" /> Clone
                    </DropdownMenuItem>
                    {!flow.isDefault && (
                      <DropdownMenuItem>
                        <Check className="mr-2 h-4 w-4" /> Set as Default
                      </DropdownMenuItem>
                    )}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            )
          })}
        </div>
      </div>

      <div className="bg-background/50 mt-auto border-t p-3 backdrop-blur-sm">
        <Button className="w-full gap-2" onClick={() => navigate('/flows/builder')}>
          <Plus className="h-4 w-4" /> Create Flow
        </Button>
      </div>
    </div>
  )
}
