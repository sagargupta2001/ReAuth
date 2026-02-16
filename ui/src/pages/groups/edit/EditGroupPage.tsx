import { useEffect } from 'react'

import { Loader2, Settings, ShieldCheck, Users } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useGroup } from '@/features/group/api/useGroup'
import { GroupHeader } from '@/features/group/components/GroupHeader'
import { GroupMembersTab } from '@/features/group/components/GroupMembersTab'
import { GroupRolesTab } from '@/features/group/components/GroupRolesTab'
import { GroupSettingsTab } from '@/features/group/components/GroupSettingsTab'

export function EditGroupPage() {
  const { groupId, tab } = useParams<{ groupId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const { data: group, isLoading, isError } = useGroup(groupId!)

  const validTabs = ['settings', 'members', 'roles']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'settings'

  useEffect(() => {
    !tab && navigate(`/groups/${groupId}/settings`, { replace: true })
  }, [tab, groupId, navigate])

  const handleTabChange = (newTab: string) => navigate(`/groups/${groupId}/${newTab}`)

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Group...</p>
      </div>
    )
  }

  if (isError || !group) {
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Group not found.</p>
        <Button variant="outline" onClick={() => navigate('/groups')}>
          Go Back
        </Button>
      </div>
    )
  }

  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden p-6">
      <div className="shrink-0">
        <GroupHeader group={group} />
      </div>

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
          <TabsList className="gap-6 bg-transparent p-0">
            <TabsTrigger value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>

            <TabsTrigger value="members" className="tab-trigger-styles">
              <Users className="mr-2 h-4 w-4" /> Members
            </TabsTrigger>

            <TabsTrigger value="roles" className="tab-trigger-styles">
              <ShieldCheck className="mr-2 h-4 w-4" /> Roles
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 h-full w-full">
            <GroupSettingsTab group={group} />
          </TabsContent>

          <TabsContent value="members" className="mt-0 h-full w-full p-6">
            <GroupMembersTab groupId={group.id} />
          </TabsContent>

          <TabsContent value="roles" className="mt-0 h-full w-full p-6">
            <GroupRolesTab groupId={group.id} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
