import { Activity, UserPlus, Users } from 'lucide-react'

import { useUserStats } from '@/features/user/api/useUserStats'
import { MetricCard } from '@/shared/ui/metric-card'

export function UserStatsCards() {
  const { data } = useUserStats()

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title="Total Users"
        value={(data?.total ?? 0).toLocaleString()}
        description="Registered in this realm"
        icon={Users}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Active (24h)"
        value={(data?.active_last_24h ?? 0).toLocaleString()}
        description="Signed in within the last 24 hours"
        icon={Activity}
        accentClassName="from-emerald-500/20 via-emerald-500/10 to-transparent"
        iconClassName="bg-emerald-500/15 text-emerald-500 ring-emerald-500/20"
        barClassName="bg-emerald-500"
      />
      <MetricCard
        title="New This Week"
        value={(data?.new_this_week ?? 0).toLocaleString()}
        description="Registered in the past 7 days"
        icon={UserPlus}
        accentClassName="from-violet-500/20 via-violet-500/10 to-transparent"
        iconClassName="bg-violet-500/15 text-violet-500 ring-violet-500/20"
        barClassName="bg-violet-500"
      />
    </div>
  )
}
