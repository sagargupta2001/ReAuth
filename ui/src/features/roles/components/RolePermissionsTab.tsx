import { useMemo, useState, useEffect } from 'react'
import { Search, Loader2 } from 'lucide-react'

import { Badge } from '@/components/badge'
import { Card, CardContent } from '@/components/card'
import { Checkbox } from '@/components/checkbox'
import { Input } from '@/components/input'
import { ScrollArea } from '@/components/scroll-area'
import { Switch } from '@/components/switch'
import { cn } from '@/lib/utils'
import { useScrollSpy } from '@/shared/hooks/useScrollSpy'

// Import extracted API hooks (Solid/FSD)
import { usePermissions } from '@/features/roles/api/usePermissions'
import { useRolePermissions, useManagePermissions } from '@/features/roles/api/useManagePermissions'

interface RolePermissionsTabProps {
  roleId: string
}

export function RolePermissionsTab({ roleId }: RolePermissionsTabProps) {
  const [search, setSearch] = useState('')

  // 1. Fetch Data via Hooks
  const { data: permissionGroups = [], isLoading: isLoadingDefs } = usePermissions()
  const { data: assignedPermissions = [], isLoading: isLoadingAssigned } = useRolePermissions(roleId)

  // 2. Mutations
  const { toggleMutation, bulkMutation } = useManagePermissions(roleId)

  // 3. Scroll Spy
  // [FIX] We map the IDs only when data exists.
  // The hook handles defaulting to the first ID internally.
  const sectionIds = useMemo(() => permissionGroups.map(r => r.id), [permissionGroups])

  const {
    activeId: activeSection,
    containerRef,
    onScroll,
    scrollToSection,
    registerSection
  } = useScrollSpy(sectionIds, { offset: 100 })

  // 4. Filter Logic
  const filteredResources = useMemo(() => {
    if (!search) return permissionGroups
    return permissionGroups.map((res) => ({
      ...res,
      permissions: res.permissions.filter(
        (p) =>
          p.name.toLowerCase().includes(search.toLowerCase()) ||
          p.id.toLowerCase().includes(search.toLowerCase()),
      ),
    })).filter((res) => res.permissions.length > 0)
  }, [search, permissionGroups])

  if (isLoadingDefs || isLoadingAssigned) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground gap-2">
         <Loader2 className="h-6 w-6 animate-spin" />
         <span>Loading Permissions...</span>
      </div>
    )
  }

  return (
    <div className="flex h-full w-full overflow-hidden bg-background">
      {/* SIDEBAR */}
      <aside className="bg-muted/10 flex w-64 flex-shrink-0 flex-col border-r">
        <div className="border-b p-4">
          <div className="relative">
            <Search className="text-muted-foreground absolute left-2.5 top-2.5 h-4 w-4" />
            <Input
              placeholder="Filter..."
              className="h-9 pl-9"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>
        <ScrollArea className="flex-1">
          <div className="space-y-1 p-3">
            {filteredResources.map((resource) => (
              <button
                key={resource.id}
                onClick={() => scrollToSection(resource.id)}
                className={cn(
                  'group flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm font-medium transition-colors',
                  activeSection === resource.id
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                )}
              >
                <span>{resource.label}</span>
                {activeSection === resource.id && (
                  <div className="bg-primary h-1.5 w-1.5 rounded-full" />
                )}
              </button>
            ))}
          </div>
        </ScrollArea>
      </aside>

      {/* CONTENT AREA */}
      <div
        ref={containerRef}
        onScroll={onScroll}
        className="flex-1 space-y-10 overflow-y-auto scroll-smooth p-8 relative"
      >
        {filteredResources.map((resource) => {
          const resourcePermIds = resource.permissions.map((p) => p.id)
          const assignedCount = resourcePermIds.filter((id) =>
            assignedPermissions.includes(id),
          ).length
          const isAllSelected = assignedCount === resourcePermIds.length && resourcePermIds.length > 0
          const isIndeterminate = assignedCount > 0 && assignedCount < resourcePermIds.length

          return (
            <div
              key={resource.id}
              ref={registerSection(resource.id)}
              className="scroll-mt-6"
            >
              <div className="mb-2 flex items-start justify-between">
                <div>
                  <h3 className="flex items-center gap-2 text-lg font-semibold">
                    {resource.label}
                    <Badge variant="outline" className="text-muted-foreground text-xs font-normal">
                      {assignedCount} / {resource.permissions.length}
                    </Badge>
                  </h3>
                  <p className="text-muted-foreground mt-1 text-sm">{resource.description}</p>
                </div>
                <div className="bg-muted/30 flex items-center space-x-2 rounded-md border px-3 py-1.5">
                  <Checkbox
                    id={`select-all-${resource.id}`}
                    checked={isAllSelected ? true : isIndeterminate ? 'indeterminate' : false}
                    onCheckedChange={(c) =>
                      bulkMutation.mutate({ permissions: resourcePermIds, action: c ? 'add' : 'remove' })
                    }
                  />
                  <label htmlFor={`select-all-${resource.id}`} className="cursor-pointer select-none text-sm font-medium">
                    Select All
                  </label>
                </div>
              </div>

              <Card className="border-muted-foreground/20 mt-4 overflow-hidden shadow-sm">
                <CardContent className="divide-y p-0">
                  {resource.permissions.map((perm) => (
                    <div key={perm.id} className="hover:bg-muted/5 flex items-center justify-between p-4 transition-colors">
                      <div className="flex-1 pr-4">
                        <div className="flex items-center gap-2">
                          <p className="text-foreground text-sm font-medium">{perm.name}</p>
                          <span className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 font-mono text-[10px]">
                            {perm.id}
                          </span>
                        </div>
                        <p className="text-muted-foreground mt-0.5 text-xs">{perm.description}</p>
                      </div>
                      <Switch
                        checked={assignedPermissions.includes(perm.id)}
                        onCheckedChange={(c) =>
                          toggleMutation.mutate({ permission: perm.id, action: c ? 'add' : 'remove' })
                        }
                      />
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>
          )
        })}
        <div className="h-20" />
      </div>
    </div>
  )
}
