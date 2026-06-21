import type { ReactNode } from 'react'

import { ThemeSummaryPanel } from './ThemeSummaryPanel'

interface ThemeTabLayoutProps {
  themeId: string
  children: ReactNode
}

/**
 * Two-column detail layout shared by theme tabs: page content on the left, the
 * sticky {@link ThemeSummaryPanel} on the right. Mirrors UserTabLayout so flow,
 * theme, and user detail pages stay structurally consistent.
 */
export function ThemeTabLayout({ themeId, children }: ThemeTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <ThemeSummaryPanel themeId={themeId} />
      </aside>
    </div>
  )
}
