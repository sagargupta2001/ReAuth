import { AppWindow, KeyRound, Layers } from 'lucide-react'

import { useRoleStats } from '@/features/roles/api/useRoleStats'
import { MetricCard } from '@/shared/ui/metric-card'

export function RoleStatsCards() {
  const { data } = useRoleStats()

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title="Total Roles"
        value={(data?.total ?? 0).toLocaleString()}
        description="Roles defined in this realm"
        icon={KeyRound}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Composite Roles"
        value={(data?.composite ?? 0).toLocaleString()}
        description="Roles that bundle other roles"
        icon={Layers}
        accentClassName="from-violet-500/20 via-violet-500/10 to-transparent"
        iconClassName="bg-violet-500/15 text-violet-500 ring-violet-500/20"
        barClassName="bg-violet-500"
      />
      <MetricCard
        title="Client Roles"
        value={(data?.client ?? 0).toLocaleString()}
        description="Scoped to a specific client"
        icon={AppWindow}
        accentClassName="from-amber-500/20 via-amber-500/10 to-transparent"
        iconClassName="bg-amber-500/15 text-amber-500 ring-amber-500/20"
        barClassName="bg-amber-500"
      />
    </div>
  )
}
