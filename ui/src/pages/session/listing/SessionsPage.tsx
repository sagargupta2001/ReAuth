import { SessionsTable } from '@/features/session/listing/SessionsTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function SessionsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Active Sessions</h2>
          <p className="text-muted-foreground">
            View and manage currently logged-in sessions across this realm.
          </p>
        </div>
      </div>
      <SessionsTable />
    </Main>
  )
}
