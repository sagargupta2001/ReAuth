import { ClientStatsCards } from '@/features/client/components/ClientStatsCards.tsx'
import { ClientsTable } from '@/features/client/components/ClientsTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function ClientsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <ClientStatsCards />
      <ClientsTable />
    </Main>
  )
}
