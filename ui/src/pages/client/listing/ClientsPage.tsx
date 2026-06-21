import { ClientsTable } from '@/features/client/components/ClientsTable.tsx'
import { ClientsPrimaryButtons } from '@/features/client/components/ClientsPrimaryButtons.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function ClientsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <ClientsPrimaryButtons />
      </div>
      <ClientsTable />
    </Main>
  )
}
