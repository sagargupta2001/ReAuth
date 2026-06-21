import {
  Activity,
  Group as GroupIcon,
  KeyRound,
  Layers,
  Settings,
  Shield,
  ShieldCheck,
  SlidersHorizontal,
  UserRoundPen,
  Users,
} from 'lucide-react'

import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data'

import type { SegmentDef, TabDef } from '../model/types'

function firstSegment(url: string): string {
  return url.replace(/^\//, '').split('/')[0] ?? ''
}

const settingsItem = sidebarData.navMain.find((i) => i.segment === 'settings')

// Settings sub-pages become a shared quick-switch group on each settings node.
const settingsSiblings = (settingsItem?.items ?? []).map((s) => ({
  label: s.title,
  url: s.url,
}))

const registry: Record<string, SegmentDef> = {}

// Derive top-level section labels + icons from the sidebar nav (single source of truth).
for (const item of sidebarData.navMain) {
  const seg = firstSegment(item.url)
  if (!seg) continue
  registry[seg] = { label: item.title, icon: item.icon }
}

// Settings has no index page → render as plain text; its children quick-switch.
if (registry['settings']) {
  registry['settings'] = { ...registry['settings'], noLink: true }
}
for (const sub of settingsItem?.items ?? []) {
  const seg = sub.url.split('/').filter(Boolean).pop()
  if (!seg) continue
  registry[seg] = { label: sub.title, siblings: settingsSiblings }
}

// Manual supplements for sub-routes not represented in the sidebar nav.
const supplements: Record<string, SegmentDef> = {
  webhooks: { label: 'Webhook', skip: true }, // /events/webhooks/:id structural passthrough
  new: { label: 'Create' },
  invitations: { label: 'Invitations' },
}

/**
 * Maps a literal URL path segment to how it should appear in the breadcrumb.
 * Add a route to the breadcrumb = add (or rely on the derived) entry here.
 */
export const SEGMENT_REGISTRY: Record<string, SegmentDef> = {
  ...registry,
  ...supplements,
}

/**
 * Tab sets for detail pages, keyed by the owning section segment (the segment
 * immediately before the entity id). Mirrors each page's in-app tab triggers so
 * the breadcrumb's tab dropdown stays in sync. Resolving a tail segment as a tab
 * takes precedence over {@link SEGMENT_REGISTRY}, which avoids tab slugs like
 * "roles"/"settings" accidentally inheriting top-level section icons.
 */
export const TAB_GROUPS: Record<string, TabDef[]> = {
  users: [
    { slug: 'profile', label: 'Profile', icon: UserRoundPen },
    { slug: 'roles', label: 'Roles', icon: ShieldCheck },
    { slug: 'credentials', label: 'Credentials', icon: KeyRound },
    { slug: 'settings', label: 'Settings', icon: Settings },
  ],
  roles: [
    { slug: 'settings', label: 'Settings', icon: Settings },
    { slug: 'permissions', label: 'Permissions', icon: ShieldCheck },
    { slug: 'composites', label: 'Composites', icon: Layers },
    { slug: 'members', label: 'Members', icon: Users },
  ],
  groups: [
    { slug: 'settings', label: 'Settings', icon: Settings },
    { slug: 'members', label: 'Members', icon: Users },
    { slug: 'roles', label: 'Roles', icon: ShieldCheck },
    { slug: 'children', label: 'Child Groups', icon: GroupIcon },
  ],
  clients: [
    { slug: 'settings', label: 'Settings', icon: Settings },
    { slug: 'roles', label: 'Roles', icon: Shield },
    { slug: 'advanced', label: 'Advanced', icon: Activity },
  ],
  // /events/webhooks/:id/:tab — owning section is "webhooks".
  webhooks: [
    { slug: 'configure', label: 'Configure', icon: SlidersHorizontal },
    { slug: 'deliveries', label: 'Deliveries', icon: Activity },
    { slug: 'settings', label: 'Settings', icon: Settings },
  ],
}
