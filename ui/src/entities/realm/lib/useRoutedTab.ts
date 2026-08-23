import { useEffect } from 'react'
import { useParams } from 'react-router-dom'

import { useRealmNavigate } from './navigation.logic'

interface RoutedTabOptions<T extends string> {
  /** Valid tab slugs, in display order. The first is the default. */
  tabs: readonly T[]
  /** Realm-relative path the tab is appended to, e.g. `/themes/abc123`. */
  basePath: string
  /**
   * Skip the redirect while the entity id is still unknown, so we do not
   * navigate to a half-built path on the first render.
   */
  enabled?: boolean
}

interface RoutedTab<T extends string> {
  activeTab: T
  onTabChange: (tab: string) => void
}

/**
 * Backs a detail page's tab strip with the `:tab?` route segment, so each tab is
 * a real URL that can be linked, bookmarked, and reached with the back button.
 *
 * Lives here rather than in `shared` because it depends on realm-scoped
 * navigation. Every detail page repeated this same param/validate/redirect
 * block; the theme page was the odd one out, holding its tab in `useState`.
 */
export function useRoutedTab<T extends string>({
  tabs,
  basePath,
  enabled = true,
}: RoutedTabOptions<T>): RoutedTab<T> {
  const { tab } = useParams<{ tab?: string }>()
  const navigate = useRealmNavigate()

  const fallback = tabs[0]
  const isKnown = (value: string | undefined): value is T =>
    Boolean(value) && (tabs as readonly string[]).includes(value as string)
  const activeTab = isKnown(tab) ? tab : fallback

  useEffect(() => {
    if (!enabled) return
    // Also canonicalises an unknown slug, rather than showing the default tab's
    // content under a URL that does not describe it.
    if (!isKnown(tab)) {
      navigate(`${basePath}/${fallback}`, { replace: true })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `tabs` is a literal
  }, [tab, basePath, fallback, enabled])

  return {
    activeTab,
    onTabChange: (next: string) => navigate(`${basePath}/${next}`),
  }
}
