import { UserStatsCards } from '@/features/user/components/UserStatsCards'
import { UsersTable } from '@/features/user/components/UsersTable'
import { Main } from '@/widgets/Layout/Main'

export function UsersPage() {
  return (
    <Main className="flex flex-1 flex-col gap-6 p-6">
      <UserStatsCards />
      <UsersTable />
    </Main>
  )
}
