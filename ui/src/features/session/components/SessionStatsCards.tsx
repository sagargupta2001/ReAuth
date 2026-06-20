import { Activity, MonitorSmartphone, Users } from 'lucide-react'

import { useSessionStats } from '@/features/session/api/useSessionStats'
import { MetricCard } from '@/shared/ui/metric-card'

export function SessionStatsCards() {
  const { data } = useSessionStats()

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title="Active Sessions"
        value={(data?.total_active ?? 0).toLocaleString()}
        description="Live sessions in this realm"
        icon={MonitorSmartphone}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Unique Users"
        value={(data?.unique_users ?? 0).toLocaleString()}
        description="Distinct users currently signed in"
        icon={Users}
        accentClassName="from-emerald-500/20 via-emerald-500/10 to-transparent"
        iconClassName="bg-emerald-500/15 text-emerald-500 ring-emerald-500/20"
        barClassName="bg-emerald-500"
      />
      <MetricCard
        title="Active (24h)"
        value={(data?.active_last_24h ?? 0).toLocaleString()}
        description="Used within the last 24 hours"
        icon={Activity}
        accentClassName="from-violet-500/20 via-violet-500/10 to-transparent"
        iconClassName="bg-violet-500/15 text-violet-500 ring-violet-500/20"
        barClassName="bg-violet-500"
      />
    </div>
  )
}
