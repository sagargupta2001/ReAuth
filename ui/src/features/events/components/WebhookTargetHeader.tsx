import { Webhook } from 'lucide-react'

import { Badge } from '@/components/badge'
import type { WebhookEndpoint } from '@/entities/events/model/types'

interface WebhookTargetHeaderProps {
  endpoint: WebhookEndpoint
}

export function WebhookTargetHeader({ endpoint }: WebhookTargetHeaderProps) {
  const isActive = endpoint.status === 'active'
  const title = endpoint.name || endpoint.url || 'Webhook endpoint'

  return (
    <header className="bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-20 flex h-16 shrink-0 items-center px-6 backdrop-blur">
      <div className="flex min-w-0 items-center gap-4">
        <div className="bg-primary/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg">
          <Webhook className="text-primary h-5 w-5" />
        </div>

        <div className="flex min-w-0 flex-col">
          <div className="flex min-w-0 items-center gap-2">
            <h1 className="text-foreground truncate text-lg font-bold tracking-tight">{title}</h1>
            <Badge variant={isActive ? 'success' : 'destructive'} className="shrink-0 text-[10px]">
              {isActive ? 'Active' : 'Disabled'}
            </Badge>
          </div>
          <span className="text-muted-foreground truncate text-sm">
            Securely deliver signed ReAuth events to this endpoint.
          </span>
        </div>
      </div>
    </header>
  )
}
