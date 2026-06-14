import { GroupExplorer } from '@/widgets/groups/GroupExplorer'
import { Main } from '@/widgets/Layout/Main.tsx'

export function GroupsPage() {
  return (
    <Main fixed className="flex flex-1 flex-col p-0">
      <GroupExplorer />
    </Main>
  )
}
