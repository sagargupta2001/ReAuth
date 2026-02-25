import { useMemo } from 'react'

import {
  Activity,
  AppWindow,
  Database,
  Laptop,
  Moon,
  ScrollText,
  Settings,
  ShieldPlus,
  Sun,
  Trash2,
  UserPlus,
  Zap,
} from 'lucide-react'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data'

import type { OmniStaticItem } from './omniTypes'

export function useOmniStaticItems() {
  const realm = useActiveRealm()

  return useMemo<OmniStaticItem[]>(() => {
    const items: OmniStaticItem[] = []

    const navItems = sidebarData.navMain
      .filter((item) => !item.items)
      .map((item) => ({
        id: `nav.${item.title.toLowerCase()}`,
        label: item.title,
        group: 'Navigation' as const,
        kind: 'link' as const,
        icon: item.icon,
        href: `/${realm}${item.url === '/' ? '' : item.url}`,
        keywords: ['navigate', item.title.toLowerCase()],
      }))

    items.push(...navItems)

    items.push(
      {
        id: 'action.create-user',
        label: 'Create User',
        group: 'Suggested Actions',
        kind: 'link',
        icon: UserPlus,
        href: `/${realm}/users/new`,
        suggested: true,
      },
      {
        id: 'action.create-webhook',
        label: 'Create Webhook Endpoint',
        group: 'Suggested Actions',
        kind: 'link',
        icon: Zap,
        href: `/${realm}/events?tab=webhooks`,
        hash: 'create-webhook',
        suggested: true,
        keywords: ['webhook', 'event', 'routing', 'endpoint'],
      },
      {
        id: 'action.create-client',
        label: 'Create Client',
        group: 'Suggested Actions',
        kind: 'link',
        icon: AppWindow,
        href: `/${realm}/clients/new`,
        suggested: true,
      },
      {
        id: 'action.create-role',
        label: 'Create Role',
        group: 'Suggested Actions',
        kind: 'link',
        icon: ShieldPlus,
        href: `/${realm}/roles/new`,
        suggested: true,
      },
      {
        id: 'action.theme-light',
        label: 'Theme: Light',
        group: 'Suggested Actions',
        kind: 'action',
        icon: Sun,
        actionId: 'theme.light',
        suggested: true,
      },
      {
        id: 'action.theme-dark',
        label: 'Theme: Dark',
        group: 'Suggested Actions',
        kind: 'action',
        icon: Moon,
        actionId: 'theme.dark',
        suggested: true,
      },
      {
        id: 'action.theme-system',
        label: 'Theme: System',
        group: 'Suggested Actions',
        kind: 'action',
        icon: Laptop,
        actionId: 'theme.system',
        suggested: true,
      },
    )

    items.push(
      {
        id: 'setting.realm-name',
        label: 'Realm Name',
        description: 'Rename the realm and update its URL',
        group: 'Settings',
        kind: 'setting',
        icon: Settings,
        href: `/${realm}/settings/general`,
        hash: 'realm-name',
        keywords: ['general', 'name', 'realm'],
      },
      {
        id: 'setting.registration-enabled',
        label: 'Enable User Registration',
        description: 'Toggle the registration flow for this realm',
        group: 'Settings',
        kind: 'toggle',
        icon: UserPlus,
        href: `/${realm}/settings/general`,
        hash: 'realm-registration',
        toggleId: 'registration',
        keywords: ['registration', 'signup', 'users'],
      },
      {
        id: 'setting.access-token-ttl',
        label: 'Access Token Lifespan',
        description: 'Access token TTL in seconds',
        group: 'Settings',
        kind: 'setting',
        icon: Zap,
        href: `/${realm}/settings/token`,
        hash: 'token-access-ttl',
        keywords: ['token', 'access', 'ttl'],
      },
      {
        id: 'setting.refresh-token-ttl',
        label: 'SSO Session Idle',
        description: 'Refresh token idle timeout in seconds',
        group: 'Settings',
        kind: 'setting',
        icon: Zap,
        href: `/${realm}/settings/token`,
        hash: 'token-refresh-ttl',
        keywords: ['token', 'refresh', 'sso'],
      },
    )

    items.push(
      {
        id: 'nav.event-routing',
        label: 'Event Routing',
        description: 'Manage webhooks and plugin deliveries',
        group: 'Navigation',
        kind: 'link',
        icon: Zap,
        href: `/${realm}/events?tab=webhooks`,
        keywords: ['events', 'webhooks', 'plugins', 'routing'],
      },
      {
        id: 'nav.event-routing-webhooks',
        label: 'Event Routing — HTTP Webhooks',
        description: 'Manage webhook endpoints and subscriptions',
        group: 'Navigation',
        kind: 'link',
        icon: Zap,
        href: `/${realm}/events?tab=webhooks`,
        keywords: ['events', 'webhooks', 'routing', 'http'],
      },
      {
        id: 'nav.event-routing-plugins',
        label: 'Event Routing — gRPC Plugins',
        description: 'Manage plugin delivery targets',
        group: 'Navigation',
        kind: 'link',
        icon: Activity,
        href: `/${realm}/events?tab=plugins`,
        keywords: ['events', 'plugins', 'routing', 'grpc'],
      },
      {
        id: 'observability.logs',
        label: 'Log Explorer',
        description: 'Inspect audit logs and events',
        group: 'Observability',
        kind: 'link',
        icon: ScrollText,
        href: `/${realm}/logs?tab=logs`,
        keywords: ['observability', 'logs', 'events', 'audit'],
      },
      {
        id: 'observability.traces',
        label: 'Traces Explorer',
        description: 'Drill into request traces',
        group: 'Observability',
        kind: 'link',
        icon: Activity,
        href: `/${realm}/logs?tab=traces`,
        keywords: ['observability', 'traces', 'latency'],
      },
      {
        id: 'observability.cache',
        label: 'Cache Manager',
        description: 'View cache metrics and namespaces',
        group: 'Observability',
        kind: 'link',
        icon: Database,
        href: `/${realm}/logs?tab=cache`,
        keywords: ['observability', 'cache', 'flush'],
      },
      {
        id: 'danger.clear-logs',
        label: 'Clear All Logs',
        description: 'Remove stored log entries',
        group: 'Danger Zone',
        kind: 'action',
        icon: Trash2,
        actionId: 'observability.clear-logs',
        href: `/${realm}/logs`,
        hash: 'logs-danger-zone',
        keywords: ['danger', 'logs', 'clear'],
      },
      {
        id: 'danger.clear-traces',
        label: 'Clear All Traces',
        description: 'Remove stored traces',
        group: 'Danger Zone',
        kind: 'action',
        icon: Trash2,
        actionId: 'observability.clear-traces',
        href: `/${realm}/logs?tab=traces`,
        hash: 'traces-danger-zone',
        keywords: ['danger', 'traces', 'clear'],
      },
      {
        id: 'danger.flush-cache',
        label: 'Flush Cache',
        description: 'Purge all cache namespaces',
        group: 'Danger Zone',
        kind: 'action',
        icon: Database,
        actionId: 'observability.flush-cache',
        href: `/${realm}/logs?tab=cache`,
        hash: 'cache-danger-zone',
        keywords: ['danger', 'cache', 'flush'],
      },
    )

    return items
  }, [realm])
}
