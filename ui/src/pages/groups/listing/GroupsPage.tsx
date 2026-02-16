import { GroupsPrimaryButtons } from '@/features/group/components/GroupsPrimaryButtons'
import { GroupsTable } from '@/features/group/components/GroupsTable'
import { Main } from '@/widgets/Layout/Main.tsx'

export function GroupsPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 p-12 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Groups</h2>
          <p className="text-muted-foreground">Organize users and roles into reusable groups.</p>
        </div>
        <GroupsPrimaryButtons />
      </div>
      <GroupsTable />
    </Main>
  )
}
