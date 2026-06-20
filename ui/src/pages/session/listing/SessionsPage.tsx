import { SessionsTable } from '@/features/session/components/SessionsTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function SessionsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <SessionsTable />
    </Main>
  )
}
