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
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/alert-dialog'
import { useOmniStaticItems } from '@/features/Search/model/useOmniStaticItems'
import { useSearch } from '@/features/Search/model/searchContext'
import type { OmniInspectorItem, OmniStaticItem } from '@/features/Search/model/omniTypes'
import { buildHaystack, rankItems } from '@/features/Search/model/omniRanking'
import { useRecentOmniItems } from '@/features/Search/model/useRecentOmniItems'
import { useDebouncedValue } from '@/shared/hooks/useDebouncedValue'
import { cn } from '@/lib/utils'
import { toast } from 'sonner'
import { apiClient } from '@/shared/api/client'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import type { User } from '@/entities/user/model/types'
import type { OidcClient } from '@/entities/oidc/model/types'
import type { Role } from '@/features/roles/api/useRoles'
import type { Group as RealmGroup } from '@/entities/group/model/types'

const groupOrder = [
  'Suggested Actions',
  'Settings',
  'Observability',
  'Danger Zone',
  'Navigation',
] as const
const dynamicGroupOrder = ['Users', 'Clients', 'Roles', 'Groups', 'Flows'] as const

type StaticEntry = {
  item: OmniStaticItem
  value: string
  inspector: OmniInspectorItem
}

type DangerActionConfig = {
  actionId: string
  confirmText: string
  description: string
}

const dangerActionConfig: DangerActionConfig[] = [
  {
    actionId: 'observability.clear-logs',
    confirmText: 'CLEAR',
    description: 'This deletes all stored logs from telemetry storage.',
  },
  {
    actionId: 'observability.clear-traces',
    confirmText: 'CLEAR',
    description: 'This deletes all stored traces from telemetry storage.',
  },
  {
    actionId: 'observability.flush-cache',
    confirmText: 'CONFIRM',
    description: 'This flushes every cache namespace.',
  },
]

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
  const [dangerInput, setDangerInput] = React.useState('')
  const [dangerAction, setDangerAction] = React.useState<{
    actionId: string
    label: string
    confirmText: string
    description: string
    href?: string
    hash?: string
  } | null>(null)
  const { recordSelection, recencyMap } = useRecentOmniItems()
  const registrationFlowIdRef = React.useRef<string | null>(null)
  const queryClient = useQueryClient()
  const inspectorDescriptionId = 'omni-inspector-description'
  const resultsId = 'omni-results'
  const inspectorId = 'omni-inspector'
  const dangerConfigMap = React.useMemo(
    () => new Map(dangerActionConfig.map((config) => [config.actionId, config])),
    [],
  )

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

  const executeAction = React.useCallback(
    (actionId?: string) => {
      if (!actionId) return
      if (actionId === 'theme.light') setTheme('light')
      if (actionId === 'theme.dark') setTheme('dark')
      if (actionId === 'theme.system') setTheme('system')
      if (actionId === 'observability.clear-logs') {
        void apiClient
          .post<{ deleted: number }>('/api/system/observability/logs/clear', {})
          .then((data) => {
            toast.success(`Logs cleared: ${data.deleted}`)
          })
          .catch(() => toast.error('Failed to clear logs'))
      }
      if (actionId === 'observability.clear-traces') {
        void apiClient
          .post<{ deleted: number }>('/api/system/observability/traces/clear', {})
          .then((data) => {
            toast.success(`Traces cleared: ${data.deleted}`)
          })
          .catch(() => toast.error('Failed to clear traces'))
      }
      if (actionId === 'observability.flush-cache') {
        void apiClient
          .post<{ flushed: string }>('/api/system/observability/cache/flush', {})
          .then((data) => {
            toast.success(`Cache flushed: ${data.flushed}`)
          })
          .catch(() => toast.error('Failed to flush cache'))
      }
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
      setDangerAction(null)
      setDangerInput('')
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

  const rankedGroups = React.useMemo(
    () =>
      rankItems(
        dynamicResults?.groups || [],
        debouncedQuery,
        (group) => buildHaystack([group.name, group.description]),
        (group) => `group:${group.id}`,
        recencyMap,
      ),
    [dynamicResults?.groups, debouncedQuery, recencyMap],
  )

  const rankedFlows = React.useMemo(
    () =>
      rankItems(
        dynamicResults?.flows || [],
        debouncedQuery,
        (flow) => buildHaystack([flow.alias, flow.description, flow.flow_type]),
        (flow) => `flow:${flow.id}`,
        recencyMap,
      ),
    [dynamicResults?.flows, debouncedQuery, recencyMap],
  )

  const hasDynamicResults =
    rankedUsers.length +
      rankedClients.length +
      rankedRoles.length +
      rankedGroups.length +
      rankedFlows.length >
    0

  const hasStaticResults = filteredStatic.length > 0
  const showEmpty = !hasDynamicResults && !hasStaticResults && !isFetching

  const visibleGroupNames = React.useMemo(() => {
    const names: string[] = []
    groupOrder.forEach((group) => {
      if (staticGroups[group]?.length) names.push(group)
    })
    dynamicGroupOrder.forEach((group) => {
      if (group === 'Users' && rankedUsers.length) names.push(group)
      if (group === 'Clients' && rankedClients.length) names.push(group)
      if (group === 'Roles' && rankedRoles.length) names.push(group)
      if (group === 'Groups' && rankedGroups.length) names.push(group)
      if (group === 'Flows' && rankedFlows.length) names.push(group)
    })
    return names
  }, [
    staticGroups,
    rankedUsers.length,
    rankedClients.length,
    rankedRoles.length,
    rankedGroups.length,
    rankedFlows.length,
  ])

  const firstValueByGroup = React.useMemo(() => {
    const map = new Map<string, string>()
    groupOrder.forEach((group) => {
      const entry = staticGroups[group]?.[0]
      if (entry) map.set(group, entry.value)
    })
    if (rankedUsers[0]) map.set('Users', `user:${rankedUsers[0].id}`)
    if (rankedClients[0]) map.set('Clients', `client:${rankedClients[0].id}`)
    if (rankedRoles[0]) map.set('Roles', `role:${rankedRoles[0].id}`)
    if (rankedGroups[0]) map.set('Groups', `group:${rankedGroups[0].id}`)
    if (rankedFlows[0]) map.set('Flows', `flow:${rankedFlows[0].id}`)
    return map
  }, [staticGroups, rankedUsers, rankedClients, rankedRoles, rankedGroups, rankedFlows])

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
    rankedGroups.forEach((group) => {
      map.set(`group:${group.id}`, {
        kind: 'group',
        id: group.id,
        label: group.name,
        subtitle: group.description || 'Group',
        href: `/${realm}/groups/${group.id}`,
      })
    })
    rankedFlows.forEach((flow) => {
      const flowSubtitle = `${flow.flow_type}${flow.is_draft ? ' · Draft' : ''}`
      map.set(`flow:${flow.id}`, {
        kind: 'flow',
        id: flow.id,
        label: flow.alias,
        subtitle: flowSubtitle,
        description: flow.description || undefined,
        href: `/${realm}/flows/${flow.id}`,
      })
    })
    return map
  }, [staticEntries, rankedUsers, rankedClients, rankedRoles, rankedGroups, rankedFlows, realm])

  React.useEffect(() => {
    if (!selectedValue) {
      setActiveItem(null)
      return
    }
    const inspector = valueToInspector.get(selectedValue) || null
    setActiveItem(inspector)
  }, [selectedValue, valueToInspector])

  React.useEffect(() => {
    if (!open) return

    const handleKeyDown = (event: KeyboardEvent) => {
      if (!event.altKey) return
      const index = Number(event.key) - 1
      if (Number.isNaN(index) || index < 0 || index > 2) return
      const groupName = visibleGroupNames[index]
      if (!groupName) return
      const nextValue = firstValueByGroup.get(groupName)
      if (!nextValue) return
      event.preventDefault()
      setSelectedValue(nextValue)
      const groupEl = document.querySelector(`[data-omni-group="${groupName}"]`)
      groupEl?.scrollIntoView({ block: 'nearest' })
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [open, visibleGroupNames, firstValueByGroup])

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
      (rankedRoles[0] ? `role:${rankedRoles[0].id}` : '') ||
      (rankedGroups[0] ? `group:${rankedGroups[0].id}` : '') ||
      (rankedFlows[0] ? `flow:${rankedFlows[0].id}` : '')

    if (firstValue) setSelectedValue(firstValue)
  }, [open, selectedValue, staticEntries, rankedUsers, rankedClients, rankedRoles, rankedGroups, rankedFlows])

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

    rankedGroups.slice(0, 3).forEach((group) => {
      void queryClient.prefetchQuery({
        queryKey: ['group', realm, group.id],
        queryFn: () =>
          apiClient.get<RealmGroup>(`/api/realms/${realm}/rbac/groups/${group.id}`),
        staleTime: 30_000,
      })
    })
  }, [canShowDynamic, rankedUsers, rankedClients, rankedRoles, rankedGroups, queryClient, realm])

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
            <div>
              <a
                href={`#${resultsId}`}
                className="focus-visible:ring-ring focus-visible:ring-offset-background sr-only inline-flex items-center rounded-md bg-background px-3 py-2 text-sm font-medium shadow focus:not-sr-only focus-visible:ring-2 focus-visible:ring-offset-2"
              >
                Skip to results
              </a>
              <a
                href={`#${inspectorId}`}
                className="focus-visible:ring-ring focus-visible:ring-offset-background sr-only ml-2 inline-flex items-center rounded-md bg-background px-3 py-2 text-sm font-medium shadow focus:not-sr-only focus-visible:ring-2 focus-visible:ring-offset-2"
              >
                Skip to inspector
              </a>
            </div>
            <div className="px-4 py-3">
              <CommandInput
                value={query}
                onValueChange={setQuery}
                placeholder="Search commands, settings, users, clients, groups, flows..."
                className="h-12 text-base"
                wrapperClassName="bg-muted/30 border border-input rounded-lg px-3"
                aria-label="Search"
              />
            </div>
            <div className="flex h-full min-h-0">
              <div
                id={resultsId}
                tabIndex={-1}
                className="focus-within:ring-ring/30 focus-within:ring-offset-background flex w-full flex-col rounded-lg focus-within:ring-2 focus-within:ring-offset-2 md:w-3/5"
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
                            <CommandGroup heading={groupName} data-omni-group={groupName}>
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
                                      if (item.kind === 'action') {
                                        const config = item.actionId
                                          ? dangerConfigMap.get(item.actionId)
                                          : undefined
                                        if (config && item.actionId) {
                                          setDangerAction({
                                            actionId: item.actionId,
                                            label: item.label,
                                            confirmText: config.confirmText,
                                            description: config.description,
                                            href: item.href,
                                            hash: item.hash,
                                          })
                                          setDangerInput('')
                                          return
                                        }
                                        runCommand(() => executeAction(item.actionId))
                                      }
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
                        <CommandGroup heading="Users" data-omni-group="Users">
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
                        <CommandGroup heading="Clients" data-omni-group="Clients">
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
                        <CommandGroup heading="Roles" data-omni-group="Roles">
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

                      {rankedGroups.length > 0 && <CommandSeparator />}
                      {rankedGroups.length > 0 && (
                        <CommandGroup heading="Groups" data-omni-group="Groups">
                          {rankedGroups.map((group) => (
                            <CommandEntityRow
                              key={group.id}
                              value={`group:${group.id}`}
                              kind="group"
                              primary={group.name}
                              secondary={group.description || 'Group'}
                              onSelect={() =>
                                runCommand(() => {
                                  recordSelection(`group:${group.id}`)
                                  navigate(`/${realm}/groups/${group.id}`)
                                })
                              }
                              onHighlight={() =>
                                setActiveItem({
                                  kind: 'group',
                                  id: group.id,
                                  label: group.name,
                                  subtitle: group.description || 'Group',
                                  href: `/${realm}/groups/${group.id}`,
                                })
                              }
                            />
                          ))}
                        </CommandGroup>
                      )}

                      {rankedFlows.length > 0 && <CommandSeparator />}
                      {rankedFlows.length > 0 && (
                        <CommandGroup heading="Flows" data-omni-group="Flows">
                          {rankedFlows.map((flow) => {
                            const flowSubtitle = `${flow.flow_type}${flow.is_draft ? ' · Draft' : ''}`
                            return (
                              <CommandEntityRow
                                key={flow.id}
                                value={`flow:${flow.id}`}
                                kind="flow"
                                primary={flow.alias}
                                secondary={flowSubtitle}
                                onSelect={() =>
                                  runCommand(() => {
                                    recordSelection(`flow:${flow.id}`)
                                    navigate(`/${realm}/flows/${flow.id}`)
                                  })
                                }
                                onHighlight={() =>
                                  setActiveItem({
                                    kind: 'flow',
                                    id: flow.id,
                                    label: flow.alias,
                                    subtitle: flowSubtitle,
                                    description: flow.description || undefined,
                                    href: `/${realm}/flows/${flow.id}`,
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
                id={inspectorId}
                tabIndex={-1}
                className="focus-within:ring-ring/30 focus-within:ring-offset-background hidden h-full w-2/5 border-l focus-within:ring-2 focus-within:ring-offset-2 md:block"
                role="region"
                aria-label="Inspector"
                aria-live="polite"
                aria-describedby={inspectorDescriptionId}
              >
                <PaletteInspector item={activeItem} descriptionId={inspectorDescriptionId} />
              </div>
            </div>
          </Command>
          <AlertDialog
            open={Boolean(dangerAction)}
            onOpenChange={(open) => {
              if (!open) {
                setDangerAction(null)
                setDangerInput('')
              }
            }}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {dangerAction ? `Confirm ${dangerAction.label}?` : 'Confirm action'}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {dangerAction?.description || 'This action cannot be undone.'}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <div className="space-y-2">
                <Input
                  placeholder={`Type ${dangerAction?.confirmText ?? ''} to confirm`}
                  value={dangerInput}
                  onChange={(event) => setDangerInput(event.target.value)}
                />
                <p className="text-xs text-muted-foreground">
                  {dangerAction?.confirmText
                    ? `Type ${dangerAction.confirmText} to enable this action.`
                    : 'Type the confirmation text to enable this action.'}
                </p>
              </div>
              <AlertDialogFooter className="flex flex-col gap-2 sm:flex-row sm:items-center">
                {dangerAction?.href && (
                  <Button
                    variant="secondary"
                    onClick={() => handleNavigate(dangerAction.href, dangerAction.hash)}
                  >
                    Open in Observability
                  </Button>
                )}
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction
                  className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                  onClick={() => {
                    executeAction(dangerAction?.actionId)
                    setDangerAction(null)
                    setDangerInput('')
                    setOpen(false)
                  }}
                  disabled={
                    !dangerAction ||
                    dangerInput.trim() !== dangerAction.confirmText
                  }
                >
                  Confirm
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </DialogPrimitive.Content>
      </DialogPortal>
    </Dialog>
  )
}
