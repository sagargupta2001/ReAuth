import type { ReactNode } from 'react'

import type { OidcClient } from '@/entities/oidc/model/types.ts'
import { ClientSummaryPanel } from '@/features/client/components/ClientSummaryPanel'

interface ClientTabLayoutProps {
  client: OidcClient
  children: ReactNode
}

export function ClientTabLayout({ client, children }: ClientTabLayoutProps) {
  return (
    <div className="grid min-h-full w-full items-start gap-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0">{children}</div>
      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <ClientSummaryPanel client={client} />
      </aside>
    </div>
  )
}
