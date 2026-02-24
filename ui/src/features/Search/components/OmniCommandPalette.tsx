import * as React from 'react'

import * as DialogPrimitive from '@radix-ui/react-dialog'
import { useNavigate } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'

import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/command'
import { Dialog, DialogOverlay, DialogPortal } from '@/components/dialog'
import { ScrollArea } from '@/components/scroll-area'
import { useTheme } from '@/app/providers/ThemeContext'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useCurrentRealm } from '@/features/realm/api/useRealm'
import { useUpdateRealmOptimistic } from '@/features/realm/api/useUpdateRealmOptimistic'
import { useOmniSearch } from '@/features/Search/api/useOmniSearch'
import { CommandEntityRow } from '@/features/Search/components/CommandEntityRow'
import { CommandSettingRow } from '@/features/Search/components/CommandSettingRow'
import { PaletteInspector } from '@/features/Search/components/PaletteInspector'
import { useOmniStaticItems } from '@/features/Search/model/useOmniStaticItems'
import { useSearch } from '@/features/Search/model/searchContext'
import type { OmniInspectorItem, OmniStaticItem } from '@/features/Search/model/omniTypes'
import { buildHaystack, rankItems } from '@/features/Search/model/omniRanking'
import { useRecentOmniItems } from '@/features/Search/model/useRecentOmniItems'
import { useDebouncedValue } from '@/shared/hooks/useDebouncedValue'
import { cn } from '@/lib/utils'
import { toast } from 'sonner'
import { apiClient } from '@/shared/api/client'
import type { User } from '@/entities/user/model/types'
import type { OidcClient } from '@/entities/oidc/model/types'
import type { Role } from '@/features/roles/api/useRoles'

const groupOrder = ['Suggested Actions', 'Settings', 'Navigation'] as const

type StaticEntry = {
  item: OmniStaticItem
  value: string
  inspector: OmniInspectorItem
}

function groupByGroup(items: StaticEntry[]) {
  return items.reduce((acc, item) => {
    acc[item.item.group] ||= []
    acc[item.item.group].push(item)
    return acc
  }, {} as Record<string, StaticEntry[]>)
}

export function OmniCommandPalette() {
  const { open, setOpen } = useSearch()
  const { setTheme } = useTheme()
  const navigate = useNavigate()
  const realm = useActiveRealm()
  const { data: realmData } = useCurrentRealm()
  const updateRealm = useUpdateRealmOptimistic(realmData?.id || '', realmData?.name || '')
  const [query, setQuery] = React.useState('')
  const [activeItem, setActiveItem] = React.useState<OmniInspectorItem | null>(null)
  const [selectedValue, setSelectedValue] = React.useState('')
  const { recordSelection, recencyMap } = useRecentOmniItems()
  const registrationFlowIdRef = React.useRef<string | null>(null)
  const queryClient = useQueryClient()

  const debouncedQuery = useDebouncedValue(query, 300)
  const staticItems = useOmniStaticItems()
  const filteredStatic = React.useMemo(() => {
    const base = query.trim()
      ? staticItems
      : staticItems.filter((item) => item.suggested || recencyMap.has(item.id))

    return rankItems(
      base,
      query,
      (item) => buildHaystack([item.label, item.description, ...(item.keywords || [])]),
      (item) => item.id,
      recencyMap,
    )
  }, [staticItems, query, recencyMap])

  const staticEntries = React.useMemo<StaticEntry[]>(() => {
    return filteredStatic.map((item) => {
      const value = `static:${item.id}`
      const inspector: OmniInspectorItem =
        item.kind === 'setting' || item.kind === 'toggle'
          ? {
              kind: 'setting',
              id: item.id,
              label: item.label,
              description: item.description,
              breadcrumb: 'Settings',
              href: item.href ? `${item.href}${item.hash ? `#${item.hash}` : ''}` : undefined,
            }
          : {
              kind: 'action',
              id: item.id,
              label: item.label,
              description: item.description,
              href: item.href ? `${item.href}${item.hash ? `#${item.hash}` : ''}` : undefined,
            }

      return { item, value, inspector }
    })
  }, [filteredStatic])

  const staticGroups = React.useMemo(() => groupByGroup(staticEntries), [staticEntries])

  const canShowDynamic = debouncedQuery.trim().length > 1
  const { data, isFetching } = useOmniSearch(debouncedQuery)
  const dynamicResults = canShowDynamic ? data : undefined

  React.useEffect(() => {
    if (realmData?.registration_flow_id) {
      registrationFlowIdRef.current = realmData.registration_flow_id
    }
  }, [realmData?.registration_flow_id])

  const registrationEnabled = Boolean(realmData?.registration_flow_id)

  const handleAction = React.useCallback(
    (actionId?: string) => {
      if (!actionId) return
      if (actionId === 'theme.light') setTheme('light')
      if (actionId === 'theme.dark') setTheme('dark')
      if (actionId === 'theme.system') setTheme('system')
    },
    [setTheme],
  )

  const runCommand = React.useCallback(
    (command: () => void, keepOpen = false) => {
      if (!keepOpen) setOpen(false)
      command()
    },
    [setOpen],
  )

  const handleNavigate = React.useCallback(
    (href?: string, hash?: string) => {
      if (!href) return
      const target = hash ? `${href}#${hash}` : href
      runCommand(() => navigate(target))
    },
    [navigate, runCommand],
  )

  const handleRegistrationToggle = React.useCallback(
    (enabled: boolean) => {
      if (!realmData) return
      const flowId = registrationFlowIdRef.current

      if (enabled && !flowId) {
        toast.error('No registration flow is configured for this realm.')
        return
      }

      recordSelection('setting.registration-enabled')
      updateRealm.mutate({
        registration_flow_id: enabled ? flowId : null,
      })
    },
    [realmData, recordSelection, updateRealm],
  )

  React.useEffect(() => {
    if (!open) {
      setQuery('')
      setActiveItem(null)
    }
  }, [open])

  const rankedUsers = React.useMemo(
    () =>
      rankItems(
        dynamicResults?.users || [],
        debouncedQuery,
        (user) => buildHaystack([user.username, user.id]),
        (user) => `user:${user.id}`,
        recencyMap,
      ),
    [dynamicResults?.users, debouncedQuery, recencyMap],
  )

  const rankedClients = React.useMemo(
    () =>
      rankItems(
        dynamicResults?.clients || [],
        debouncedQuery,
        (client) => buildHaystack([client.client_id, client.id]),
        (client) => `client:${client.id}`,
        recencyMap,
      ),
    [dynamicResults?.clients, debouncedQuery, recencyMap],
  )

  const rankedRoles = React.useMemo(
    () =>
      rankItems(
        dynamicResults?.roles || [],
        debouncedQuery,
        (role) => buildHaystack([role.name, role.description, role.client_id]),
        (role) => `role:${role.id}`,
        recencyMap,
      ),
    [dynamicResults?.roles, debouncedQuery, recencyMap],
  )

  const hasDynamicResults =
    rankedUsers.length + rankedClients.length + rankedRoles.length > 0

  const hasStaticResults = filteredStatic.length > 0
  const showEmpty = !hasDynamicResults && !hasStaticResults && !isFetching

  const valueToInspector = React.useMemo(() => {
    const map = new Map<string, OmniInspectorItem>()
    staticEntries.forEach((entry) => map.set(entry.value, entry.inspector))
    rankedUsers.forEach((user) =>
      map.set(`user:${user.id}`, {
        kind: 'user',
        id: user.id,
        label: user.username,
        subtitle: user.id,
        href: `/${realm}/users/${user.id}/settings`,
      }),
    )
    rankedClients.forEach((client) =>
      map.set(`client:${client.id}`, {
        kind: 'client',
        id: client.id,
        label: client.client_id,
        subtitle: client.id,
        href: `/${realm}/clients/${client.id}/settings`,
      }),
    )
    rankedRoles.forEach((role) => {
      const roleSubtitle =
        role.description ||
        (role.client_id ? `Client role · ${role.client_id}` : 'Realm role')
      map.set(`role:${role.id}`, {
        kind: 'role',
        id: role.id,
        label: role.name,
        subtitle: roleSubtitle,
        href: `/${realm}/roles/${role.id}/settings`,
      })
    })
    return map
  }, [staticEntries, rankedUsers, rankedClients, rankedRoles, realm])

  React.useEffect(() => {
    if (!selectedValue) return
    const inspector = valueToInspector.get(selectedValue)
    if (inspector) setActiveItem(inspector)
  }, [selectedValue, valueToInspector])

  React.useEffect(() => {
    if (!open) {
      setSelectedValue('')
      return
    }

    if (selectedValue) return

    const firstValue =
      staticEntries[0]?.value ||
      (rankedUsers[0] ? `user:${rankedUsers[0].id}` : '') ||
      (rankedClients[0] ? `client:${rankedClients[0].id}` : '') ||
      (rankedRoles[0] ? `role:${rankedRoles[0].id}` : '')

    if (firstValue) setSelectedValue(firstValue)
  }, [open, selectedValue, staticEntries, rankedUsers, rankedClients, rankedRoles])

  React.useEffect(() => {
    if (!canShowDynamic) return

    rankedUsers.slice(0, 3).forEach((user) => {
      void queryClient.prefetchQuery({
        queryKey: ['user', user.id],
        queryFn: () => apiClient.get<User>(`/api/realms/${realm}/users/${user.id}`),
        staleTime: 30_000,
      })
    })

    rankedClients.slice(0, 3).forEach((client) => {
      void queryClient.prefetchQuery({
        queryKey: ['client', realm, client.id],
        queryFn: () => apiClient.get<OidcClient>(`/api/realms/${realm}/clients/${client.id}`),
        staleTime: 30_000,
      })
    })

    rankedRoles.slice(0, 3).forEach((role) => {
      void queryClient.prefetchQuery({
        queryKey: ['role', realm, role.id],
        queryFn: () => apiClient.get<Role>(`/api/realms/${realm}/rbac/roles/${role.id}`),
        staleTime: 30_000,
      })
    })
  }, [canShowDynamic, rankedUsers, rankedClients, rankedRoles, queryClient, realm])

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogPortal>
        <DialogOverlay className="bg-slate-950/40 backdrop-blur-sm" />
        <DialogPrimitive.Content
          className={cn(
            'fixed left-1/2 top-1/2 z-50 w-[min(95vw,64rem)] -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-2xl border bg-background shadow-2xl',
          )}
        >
          <Command
            shouldFilter={false}
            className="flex h-[32rem] w-full flex-col bg-transparent"
            value={selectedValue}
            onValueChange={setSelectedValue}
            aria-label="Omni search command palette"
          >
            <div className="px-4 py-3">
              <CommandInput
                value={query}
                onValueChange={setQuery}
                placeholder="Search commands, settings, users, clients..."
                className="h-12 text-base"
                wrapperClassName="bg-muted/30 border border-input rounded-lg px-3"
                aria-label="Search"
              />
            </div>
            <div className="flex h-full min-h-0">
              <div
                className="flex w-full flex-col md:w-3/5"
                role="region"
                aria-label="Search results"
              >
                <CommandList className="h-full max-h-none">
                  <ScrollArea className="h-full">
                    <div className="py-2">
                      {groupOrder.map((groupName, index) => {
                        const groupItems = staticGroups[groupName] || []
                        if (!groupItems.length) return null
                        return (
                          <React.Fragment key={groupName}>
                            {index > 0 && <CommandSeparator />}
                            <CommandGroup heading={groupName}>
                              {groupItems.map((entry) => {
                                const item = entry.item

                                if (item.kind === 'setting' || item.kind === 'toggle') {
                                  return (
                                    <CommandSettingRow
                                      key={entry.value}
                                      value={entry.value}
                                      icon={item.icon}
                                      label={item.label}
                                      description={item.description}
                                      onSelect={() => {
                                        recordSelection(item.id)
                                        handleNavigate(item.href, item.hash)
                                      }}
                                      onHighlight={() => setActiveItem(entry.inspector)}
                                      toggle={
                                        item.kind === 'toggle' && item.toggleId === 'registration'
                                          ? {
                                              checked: registrationEnabled,
                                              onChange: handleRegistrationToggle,
                                              ariaLabel: item.label,
                                              disabled: updateRealm.isPending,
                                            }
                                          : undefined
                                      }
                                    />
                                  )
                                }

                                return (
                                  <CommandItem
                                    key={entry.value}
                                    value={entry.value}
                                    onSelect={() => {
                                      recordSelection(item.id)
                                      if (item.kind === 'link') handleNavigate(item.href, item.hash)
                                      if (item.kind === 'action') runCommand(() => handleAction(item.actionId))
                                    }}
                                    onFocus={() => setActiveItem(entry.inspector)}
                                    onMouseEnter={() => setActiveItem(entry.inspector)}
                                    className="py-2"
                                  >
                                    <div className="flex w-full items-center gap-3">
                                      <div className="bg-muted/60 text-muted-foreground flex h-8 w-8 items-center justify-center rounded-md">
                                        <item.icon className="h-4 w-4" />
                                      </div>
                                      <div className="flex flex-1 flex-col">
                                        <span className="text-sm font-medium">{item.label}</span>
                                        {item.description && (
                                          <span className="text-xs text-muted-foreground">
                                            {item.description}
                                          </span>
                                        )}
                                      </div>
                                    </div>
                                  </CommandItem>
                                )
                              })}
                            </CommandGroup>
                          </React.Fragment>
                        )
                      })}

                      {rankedUsers.length > 0 && <CommandSeparator />}
                      {rankedUsers.length > 0 && (
                        <CommandGroup heading="Users">
                          {rankedUsers.map((user) => (
                            <CommandEntityRow
                              key={user.id}
                              value={`user:${user.id}`}
                              kind="user"
                              primary={user.username}
                              secondary={user.id}
                              onSelect={() =>
                                runCommand(() => {
                                  recordSelection(`user:${user.id}`)
                                  navigate(`/${realm}/users/${user.id}/settings`)
                                })
                              }
                              onHighlight={() =>
                                setActiveItem({
                                  kind: 'user',
                                  id: user.id,
                                  label: user.username,
                                  subtitle: user.id,
                                  href: `/${realm}/users/${user.id}/settings`,
                                })
                              }
                            />
                          ))}
                        </CommandGroup>
                      )}

                      {rankedClients.length > 0 && <CommandSeparator />}
                      {rankedClients.length > 0 && (
                        <CommandGroup heading="Clients">
                          {rankedClients.map((client) => (
                            <CommandEntityRow
                              key={client.id}
                              value={`client:${client.id}`}
                              kind="client"
                              primary={client.client_id}
                              secondary={client.id}
                              onSelect={() =>
                                runCommand(() => {
                                  recordSelection(`client:${client.id}`)
                                  navigate(`/${realm}/clients/${client.id}/settings`)
                                })
                              }
                              onHighlight={() =>
                                setActiveItem({
                                  kind: 'client',
                                  id: client.id,
                                  label: client.client_id,
                                  subtitle: client.id,
                                  href: `/${realm}/clients/${client.id}/settings`,
                                })
                              }
                            />
                          ))}
                        </CommandGroup>
                      )}

                      {rankedRoles.length > 0 && <CommandSeparator />}
                      {rankedRoles.length > 0 && (
                        <CommandGroup heading="Roles">
                          {rankedRoles.map((role) => {
                            const roleSubtitle =
                              role.description ||
                              (role.client_id
                                ? `Client role · ${role.client_id}`
                                : 'Realm role')

                            return (
                              <CommandEntityRow
                                key={role.id}
                                value={`role:${role.id}`}
                                kind="role"
                                primary={role.name}
                                secondary={roleSubtitle}
                                onSelect={() =>
                                  runCommand(() => {
                                    recordSelection(`role:${role.id}`)
                                    navigate(`/${realm}/roles/${role.id}/settings`)
                                  })
                                }
                                onHighlight={() =>
                                  setActiveItem({
                                    kind: 'role',
                                    id: role.id,
                                    label: role.name,
                                    subtitle: roleSubtitle,
                                    href: `/${realm}/roles/${role.id}/settings`,
                                  })
                                }
                              />
                            )
                          })}
                        </CommandGroup>
                      )}

                      {canShowDynamic && isFetching && (
                        <div className="px-4 py-3 text-sm text-muted-foreground">Searching...</div>
                      )}
                      {showEmpty && (
                        <div className="px-4 py-6 text-center text-sm text-muted-foreground">
                          No results found.
                        </div>
                      )}
                    </div>
                  </ScrollArea>
                </CommandList>
              </div>
              <div
                className="hidden h-full w-2/5 border-l md:block"
                role="region"
                aria-label="Inspector"
                aria-live="polite"
              >
                <PaletteInspector item={activeItem} />
              </div>
            </div>
          </Command>
        </DialogPrimitive.Content>
      </DialogPortal>
    </Dialog>
  )
}
