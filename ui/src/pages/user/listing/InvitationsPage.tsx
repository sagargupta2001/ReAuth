import { InvitationStatsCards } from '@/features/invitation/components/InvitationStatsCards'
import { InvitationsTable } from '@/features/invitation/components/InvitationsTable'
import { Main } from '@/widgets/Layout/Main'

export function InvitationsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-6 p-6">
      <InvitationStatsCards />
      <InvitationsTable />
    </Main>
  )
}
