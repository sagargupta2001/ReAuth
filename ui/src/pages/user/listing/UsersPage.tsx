import { UsersTable } from '@/features/user/components/UsersTable.tsx'
import { UsersPrimaryButtons } from '@/features/user/components/UsersPrimaryButtons.tsx'
import { Main } from '@/widgets/Layout/Main.tsx'

export function UsersPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Users</h2>
          <p className="text-muted-foreground">Manage users in this realm.</p>
        </div>
        <UsersPrimaryButtons />
      </div>
      <UsersTable />
    </Main>
  )
}
