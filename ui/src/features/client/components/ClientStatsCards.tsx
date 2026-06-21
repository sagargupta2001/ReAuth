import { AppWindow, Globe, ShieldCheck } from 'lucide-react'

import { useClientStats } from '@/features/client/api/useClientStats'
import { MetricCard } from '@/shared/ui/metric-card'

export function ClientStatsCards() {
  const { data } = useClientStats()

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title="Total Clients"
        value={(data?.total ?? 0).toLocaleString()}
        description="Applications registered in this realm"
        icon={AppWindow}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Confidential Clients"
        value={(data?.confidential ?? 0).toLocaleString()}
        description="Clients that authenticate with a secret"
        icon={ShieldCheck}
        accentClassName="from-violet-500/20 via-violet-500/10 to-transparent"
        iconClassName="bg-violet-500/15 text-violet-500 ring-violet-500/20"
        barClassName="bg-violet-500"
      />
      <MetricCard
        title="Public Clients"
        value={(data?.public ?? 0).toLocaleString()}
        description="Clients without a secret (e.g. SPAs)"
        icon={Globe}
        accentClassName="from-amber-500/20 via-amber-500/10 to-transparent"
        iconClassName="bg-amber-500/15 text-amber-500 ring-amber-500/20"
        barClassName="bg-amber-500"
      />
    </div>
  )
}
