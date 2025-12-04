import { useState } from 'react'

import { ChevronRight, Plus, Search } from 'lucide-react'
import { NavLink } from 'react-router-dom'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { cn } from '@/lib/utils'

// Mock data for now - replace with useFlows() hook later
const mockFlows = [
  { id: 'browser-login', alias: 'Browser Login', description: 'Standard username/password' },
  { id: 'registration', alias: 'Registration', description: 'New user sign up' },
  { id: 'reset-credentials', alias: 'Reset Credentials', description: 'Forgot password flow' },
  { id: 'direct-grant', alias: 'Direct Grant', description: 'OAuth2 Resource Owner Password' },
]

export function FlowsSidebar() {
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()
  const [search, setSearch] = useState('')

  const filteredFlows = mockFlows.filter((f) =>
    f.alias.toLowerCase().includes(search.toLowerCase()),
  )

  return (
    <div className="bg-sidebar-accent/10 flex h-full w-[var(--sidebar-width-secondary)] flex-col border-r">
      {/* Fixed Header */}
      <div className="flex h-14 shrink-0 items-center border-b px-4">
        <h2 className="font-semibold">Authentication Flows</h2>
        <div className="bg-muted ml-auto flex h-6 w-6 items-center justify-center rounded-full text-xs font-medium">
          {filteredFlows.length}
        </div>
      </div>

      {/* Search Bar */}
      <div className="p-3 pb-0">
        <div className="relative">
          <Search className="text-muted-foreground absolute top-2.5 left-2 h-4 w-4" />
          <Input
            placeholder="Search flows..."
            className="bg-background h-9 pl-8"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>

      {/* Scrollable List */}
      <div className="flex-1 overflow-y-auto p-2">
        <div className="flex flex-col gap-1">
          {filteredFlows.map((flow) => (
            <NavLink
              key={flow.id}
              to={`/${realm}/flows/${flow.id}`}
              className={({ isActive }) =>
                cn(
                  'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground flex flex-col items-start gap-1 rounded-md p-3 text-sm transition-colors',
                  isActive
                    ? 'bg-sidebar-accent text-sidebar-accent-foreground shadow-sm'
                    : 'text-muted-foreground',
                )
              }
            >
              <div className="flex w-full items-center justify-between">
                <span className="text-foreground font-medium">{flow.alias}</span>
                <ChevronRight className="h-3 w-3 opacity-50" />
              </div>
              <span className="line-clamp-1 text-xs opacity-80">{flow.description}</span>
            </NavLink>
          ))}
        </div>
      </div>

      {/* Fixed Bottom Button */}
      <div className="bg-background/50 mt-auto border-t p-3 backdrop-blur-sm">
        <Button className="w-full gap-2" onClick={() => navigate('/flows/create')}>
          <Plus className="h-4 w-4" />
          Create Flow
        </Button>
      </div>
    </div>
  )
}
