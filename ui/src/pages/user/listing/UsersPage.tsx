import { UsersTable } from '@/features/user/components/UsersTable.tsx'
import { InvitationsTable } from '@/features/invitation/components/InvitationsTable'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { enumParam, useUrlState } from '@/shared/lib/hooks/useUrlState'
import { Main } from '@/widgets/Layout/Main.tsx'

const TAB_OPTIONS = ['all', 'invitations'] as const

export function UsersPage() {
  const [urlState, setUrlState] = useUrlState<{
    tab: (typeof TAB_OPTIONS)[number]
  }>({
    tab: enumParam(TAB_OPTIONS, 'all'),
  })

  const activeTab = urlState.tab

  return (
    <Main className="flex flex-1 flex-col gap-4 sm:gap-6 p-12">
      <h2 className="text-2xl font-bold tracking-tight">Users</h2>
      <Tabs
        value={activeTab}
        onValueChange={(value) => setUrlState({ tab: value as (typeof TAB_OPTIONS)[number] })}
      >
        <TabsList variant="line">
          <TabsTrigger variant="line" value="all">All</TabsTrigger>
          <TabsTrigger variant="line" value="invitations">Invitations</TabsTrigger>
        </TabsList>
        <TabsContent value="all" className="mt-4">
          <UsersTable />
        </TabsContent>
        <TabsContent value="invitations" className="mt-4">
          <InvitationsTable />
        </TabsContent>
      </Tabs>
    </Main>
  )
}
