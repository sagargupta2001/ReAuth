import { ClientsTable } from '@/features/client/listing/ClientsTable.tsx'
import { ClientsPrimaryButtons } from '@/features/client/listing/components/ClientsPrimaryButtons.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function ClientsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Clients</h2>
          <p className="text-muted-foreground">
            Clients are applications and services that can request authentication of a user.
          </p>
        </div>
        <ClientsPrimaryButtons />
      </div>
      <ClientsTable />
    </Main>
  )
}
