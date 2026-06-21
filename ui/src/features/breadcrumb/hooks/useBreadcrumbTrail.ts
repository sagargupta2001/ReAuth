import type { ReactNode } from 'react'

import { useLocation } from 'react-router-dom'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { sidebarData } from '@/widgets/Sidebar/config/sidebar-data'

import { SEGMENT_REGISTRY, TAB_GROUPS } from '../config/breadcrumb-config'
import { useBreadcrumbStore } from '../model/useBreadcrumbStore'
import type { BreadcrumbNode } from '../model/types'

function titleCase(value: string): string {
  return value
    .split('-')
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(' ')
}

/** Heuristic: does a segment look like an opaque entity id rather than a slug? */
function looksLikeId(segment: string): boolean {
  return /\d/.test(segment) || segment.length > 16 || /^[0-9a-f-]{8,}$/i.test(segment)
}

/** "users" -> "User" — a sensible label for an unresolved id under that section. */
function singular(segment: string): string {
  return titleCase(segment.endsWith('s') ? segment.slice(0, -1) : segment)
}

/**
 * Resolves the current location into a breadcrumb trail.
 *
 * Hybrid source: the path + {@link SEGMENT_REGISTRY} drive structure and labels,
 * while {@link useBreadcrumbStore} overrides supply dynamic names. The realm is
 * intentionally omitted (it lives in the header's left realm switcher).
 *
 * The breadcrumb doubles as the page title (pages no longer render their own), so
 * even single-level routes return one node. The realm root resolves to "Overview".
 */
export function useBreadcrumbTrail(): BreadcrumbNode[] {
  const location = useLocation()
  const realm = useActiveRealm()
  const overrides = useBreadcrumbStore((s) => s.overrides)

  const segments = location.pathname.split('/').filter(Boolean)
  const [realmSegment, ...sectionSegments] = segments
  const realmRoot = `/${realmSegment ?? realm}`

  // Realm root (the dashboard) has no section segment — surface "Overview".
  if (sectionSegments.length === 0) {
    const overview = sidebarData.navMain.find((i) => i.url === '/')
    if (!overview) return []
    return [{ id: realmRoot, label: overview.title, icon: overview.icon, isCurrent: true }]
  }

  const nodes: BreadcrumbNode[] = []
  let cumulative = realmRoot
  let prevSeg = ''
  const lastIndex = sectionSegments.length - 1

  sectionSegments.forEach((seg, i) => {
    cumulative += `/${seg}`
    const def = SEGMENT_REGISTRY[seg]
    if (def?.skip) {
      prevSeg = seg
      return
    }

    const isCurrent = i === lastIndex

    // Tail segment that follows an entity id is a tab — resolve it from the
    // owning section's tab group (precedence over the segment registry, so tab
    // slugs don't borrow a section's label/icon). Render it as a quick-switch.
    const owningSection = sectionSegments[i - 2]
    const tab = isCurrent && owningSection ? TAB_GROUPS[owningSection]?.find((t) => t.slug === seg) : undefined
    if (tab) {
      const base = cumulative.slice(0, cumulative.lastIndexOf('/'))
      nodes.push({
        id: cumulative,
        label: tab.label,
        icon: tab.icon,
        isCurrent: true,
        siblings: TAB_GROUPS[owningSection].map((t) => ({
          label: t.label,
          href: `${base}/${t.slug}`,
          icon: t.icon,
        })),
      })
      prevSeg = seg
      return
    }

    let label: ReactNode
    if (overrides[seg]) label = overrides[seg]
    else if (def) label = def.label
    else if (looksLikeId(seg)) label = singular(prevSeg || seg)
    else label = titleCase(seg)

    const navigable = !isCurrent && !def?.noLink

    nodes.push({
      id: cumulative,
      label,
      href: navigable ? cumulative : undefined,
      icon: def?.icon,
      isCurrent,
      siblings: def?.siblings?.map((s) => ({ label: s.label, href: `/${realm}${s.url}` })),
    })
    prevSeg = seg
  })

  return nodes
}
