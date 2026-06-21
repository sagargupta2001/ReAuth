import { Clock, Send, UserCheck } from 'lucide-react'

import { useInvitationStats } from '@/features/invitation/api/useInvitationStats'
import { MetricCard } from '@/shared/ui/metric-card'

export function InvitationStatsCards() {
  const { data } = useInvitationStats()

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MetricCard
        title="Total Invitations"
        value={(data?.total ?? 0).toLocaleString()}
        description="All time in this realm"
        icon={Send}
        accentClassName="from-sky-500/20 via-sky-500/10 to-transparent"
        iconClassName="bg-sky-500/15 text-sky-500 ring-sky-500/20"
        barClassName="bg-sky-500"
      />
      <MetricCard
        title="Pending"
        value={(data?.pending ?? 0).toLocaleString()}
        description="Awaiting acceptance"
        icon={Clock}
        accentClassName="from-amber-500/20 via-amber-500/10 to-transparent"
        iconClassName="bg-amber-500/15 text-amber-500 ring-amber-500/20"
        barClassName="bg-amber-500"
      />
      <MetricCard
        title="Accepted"
        value={(data?.accepted ?? 0).toLocaleString()}
        description="Successfully accepted"
        icon={UserCheck}
        accentClassName="from-emerald-500/20 via-emerald-500/10 to-transparent"
        iconClassName="bg-emerald-500/15 text-emerald-500 ring-emerald-500/20"
        barClassName="bg-emerald-500"
      />
    </div>
  )
}
