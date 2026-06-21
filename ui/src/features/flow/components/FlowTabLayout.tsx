import type { ReactNode } from 'react'

import type { FlowDraft } from '@/entities/flow/model/types'

import { FlowSummaryPanel } from './FlowSummaryPanel.tsx'

interface FlowTabLayoutProps {
  draft: FlowDraft
  children: ReactNode
}

/**
 * Two-column detail layout shared by flow tabs: page content on the left, the
 * sticky {@link FlowSummaryPanel} on the right. Mirrors UserTabLayout so flow,
 * theme, and user detail pages stay structurally consistent.
 */
export function FlowTabLayout({ draft, children }: FlowTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <FlowSummaryPanel draft={draft} />
      </aside>
    </div>
  )
}
