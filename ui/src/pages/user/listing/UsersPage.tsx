import { UsersTable } from '@/features/user/components/UsersTable.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function UsersPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <h2 className="text-2xl font-bold tracking-tight">Users</h2>
      <UsersTable />
    </Main>
  )
}
