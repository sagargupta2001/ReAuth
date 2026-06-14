import { useParams } from 'react-router-dom'

import { GroupExplorer } from '@/widgets/groups/GroupExplorer'
import { Main } from '@/widgets/Layout/Main.tsx'

export function EditGroupPage() {
  const { groupId, tab } = useParams<{ groupId: string; tab?: string }>()

  return (
    <Main fixed className="flex flex-1 flex-col p-0">
      <GroupExplorer groupId={groupId} tab={tab} />
    </Main>
  )
}
