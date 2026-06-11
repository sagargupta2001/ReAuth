import { useEffect, useMemo, useRef } from 'react'

import { useVirtualizer } from '@tanstack/react-virtual'
import { Loader2, Shield } from 'lucide-react'

import { Switch } from '@/components/switch'
import type { UserRoleRow } from '@/features/user/api/useUserRoles'
import { UserRoleAccessBadge } from '@/features/user/components/UserRoleAccessBadge'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/shared/ui/checkbox'

interface UserRolesVirtualListProps {
  roles: UserRoleRow[]
  hasNextPage: boolean
  isFetchingNextPage: boolean
  isMutating: boolean
  selectedRoleIds: Set<string>
  onFetchNextPage: () => void
  onToggleRoleSelection: (roleId: string, selected: boolean) => void
  onToggleLoadedSelection: (selected: boolean) => void
  onSetDirect: (role: UserRoleRow, checked: boolean) => void
}

export function UserRolesVirtualList({
  roles,
  hasNextPage,
  isFetchingNextPage,
  isMutating,
  selectedRoleIds,
  onFetchNextPage,
  onToggleRoleSelection,
  onToggleLoadedSelection,
  onSetDirect,
}: UserRolesVirtualListProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const rowCount = roles.length + (hasNextPage ? 1 : 0)
  const rowHeight = 56
  const emptyStateHeight = 128
  const listHeight = rowCount === 0 ? emptyStateHeight : rowCount * rowHeight
  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => rowHeight,
    overscan: 8,
  })

  const virtualItems = rowVirtualizer.getVirtualItems()
  const lastVirtualIndex = virtualItems.at(-1)?.index

  useEffect(() => {
    if (lastVirtualIndex == null) return
    if (lastVirtualIndex >= roles.length && hasNextPage && !isFetchingNextPage) {
      onFetchNextPage()
    }
  }, [hasNextPage, isFetchingNextPage, lastVirtualIndex, onFetchNextPage, roles.length])

  const selectionState = useMemo(() => {
    if (!roles.length) return false
    const selectedCount = roles.filter((role) => selectedRoleIds.has(role.id)).length
    if (selectedCount === 0) return false
    return selectedCount === roles.length ? true : 'indeterminate'
  }, [roles, selectedRoleIds])

  return (
    <div className="bg-table-background min-w-0 overflow-hidden rounded-2xl p-2">
      <div className="text-muted-foreground grid min-w-176 grid-cols-[2.25rem_minmax(10rem,1fr)_8rem_5rem] items-center gap-3  px-3 py-2 text-xs font-medium">
        <Checkbox
          checked={selectionState}
          disabled={!roles.length}
          onCheckedChange={(checked) => onToggleLoadedSelection(Boolean(checked))}
          aria-label="Select loaded roles"
        />
        <span>Role</span>
        <span>Access</span>
        <span>Direct</span>
      </div>

      <div
        ref={scrollRef}
        className="max-h-[calc(100vh-450px)] overflow-auto"
        style={{ height: `${listHeight}px` }}
      >
        {rowCount === 0 ? (
          <div className="text-muted-foreground flex h-32 items-center justify-center text-sm">
            No roles found.
          </div>
        ) : (
          <div
            className="relative min-w-176"
            style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
          >
            {virtualItems.map((virtualItem) => {
              const role = roles[virtualItem.index]

              if (!role) {
                return (
                  <div
                    key="loader"
                    className="text-muted-foreground absolute left-0 top-0 flex w-full items-center justify-center gap-2 px-3 py-4 text-sm"
                    style={{
                      height: `${virtualItem.size}px`,
                      transform: `translateY(${virtualItem.start}px)`,
                    }}
                  >
                    <Loader2 className={cn('size-4', isFetchingNextPage && 'animate-spin')} />
                    Loading more roles...
                  </div>
                )
              }

              const selected = selectedRoleIds.has(role.id)
              const isFirstRole = virtualItem.index === 0
              const isLastRole = virtualItem.index === roles.length - 1 && !hasNextPage

              return (
                <div
                  key={role.id}
                  className="absolute left-0 top-0 w-full"
                  style={{
                    height: `${virtualItem.size}px`,
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  <div
                    className={cn(
                      'grid h-14 grid-cols-[2.25rem_minmax(10rem,1fr)_8rem_5rem] items-center gap-3 border-x border-b border-border/60 bg-background px-2 transition-colors',
                      isFirstRole && 'rounded-t-2xl border-t',
                      isLastRole && 'rounded-b-2xl',
                      selected && 'ring-1 ring-primary/40',
                    )}
                  >
                    <Checkbox
                      checked={selected}
                      onCheckedChange={(checked) =>
                        onToggleRoleSelection(role.id, Boolean(checked))
                      }
                      aria-label={`Select ${role.name}`}
                    />
                    <div className="min-w-0">
                      <div className="flex min-w-0 items-center gap-2 font-medium">
                        <Shield className="text-muted-foreground size-4 shrink-0" />
                        <span className="truncate">{role.name}</span>
                      </div>
                      <p className="text-muted-foreground truncate text-xs">
                        {role.description || '-'}
                      </p>
                    </div>
                    <UserRoleAccessBadge role={role} />
                    <Switch
                      checked={role.is_direct}
                      disabled={isMutating}
                      onCheckedChange={(checked) => onSetDirect(role, checked)}
                      aria-label={`Toggle direct assignment for ${role.name}`}
                    />
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}
