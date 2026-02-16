import { GroupExplorer } from '@/widgets/groups/GroupExplorer'
import { Main } from '@/widgets/Layout/Main.tsx'

export function GroupsPage() {
  return (
    <Main fixed className="flex flex-1 flex-col gap-4 p-12 sm:gap-6">
      <GroupExplorer />
    </Main>
  )
}
