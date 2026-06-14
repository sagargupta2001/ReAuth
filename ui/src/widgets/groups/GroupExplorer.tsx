import { useEffect, useMemo } from 'react'

import { Loader2 } from 'lucide-react'

import groupHierarchyEmpty from '@/assets/group-hierarchy-empty.svg'
import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useGroup } from '@/features/group/api/useGroup'
import { GroupChildrenTab } from '@/features/group/components/GroupChildrenTab'
import { GroupHeader } from '@/features/group/components/GroupHeader'
import { GroupMembersTab } from '@/features/group/components/GroupMembersTab'
import { GroupRolesTab } from '@/features/group/components/GroupRolesTab'
import { GroupSettingsTab } from '@/features/group/components/GroupSettingsTab'

interface GroupExplorerProps {
  groupId?: string
  tab?: string
}

const validTabs = ['settings', 'members', 'roles', 'children']

export function GroupExplorer({ groupId, tab }: GroupExplorerProps) {
  const navigate = useRealmNavigate()

  const activeTab = useMemo(() => {
    if (!groupId) return 'settings'
    return validTabs.includes(tab || '') ? (tab as string) : 'settings'
  }, [groupId, tab])

  const { data: group, isLoading, isError } = useGroup(groupId || '', {
    enabled: !!groupId,
  })

  useEffect(() => {
    if (groupId && !tab) {
      navigate(`/groups/${groupId}/settings`, { replace: true })
    }
  }, [groupId, navigate, tab])

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col">
      {!groupId ? (
        <GroupExplorerPlaceholder />
      ) : isLoading ? (
        <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-2">
          <Loader2 className="h-6 w-6 animate-spin" />
          <span>Loading group...</span>
        </div>
      ) : isError || !group ? (
        <div className="text-destructive flex h-full flex-col items-center justify-center gap-2">
          <p>Group not found.</p>
          <Button variant="outline" onClick={() => navigate('/groups')}>
            Back to groups
          </Button>
        </div>
      ) : (
        <div className="flex h-full flex-col overflow-hidden">
          <GroupHeader group={group} showBack={false} />

          <Tabs
            value={activeTab}
            onValueChange={(value) => navigate(`/groups/${group.id}/${value}`)}
            className="flex flex-1 flex-col overflow-hidden"
          >
            <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
              <TabsList variant="line" className="gap-6 bg-transparent p-0">
                <TabsTrigger variant="line" value="settings" className="tab-trigger-styles">
                  Settings
                </TabsTrigger>
                <TabsTrigger variant="line" value="members" className="tab-trigger-styles">
                  Members
                </TabsTrigger>
                <TabsTrigger variant="line" value="roles" className="tab-trigger-styles">
                  Roles
                </TabsTrigger>
                <TabsTrigger variant="line" value="children" className="tab-trigger-styles">
                  Child Groups
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

              <TabsContent value="children" className="mt-0 h-full w-full p-6">
                <GroupChildrenTab groupId={group.id} />
              </TabsContent>
            </div>
          </Tabs>
        </div>
      )}
    </div>
  )
}

function GroupExplorerPlaceholder() {
  return (
    <div className="flex h-full flex-col items-center justify-center px-6 text-center">
      <div className="mb-6 aspect-square w-full max-w-[360px] overflow-hidden bg-[#05070a]">
        <img
          src={groupHierarchyEmpty}
          alt=""
          aria-hidden="true"
          className="h-full w-full object-cover"
        />
      </div>
      <div className="max-w-sm space-y-2">
        <h2 className="text-2xl font-semibold tracking-tight">Groups</h2>
        <p className="text-muted-foreground text-sm">
          Organize users and roles into hierarchical groups.
        </p>
        <p className="text-muted-foreground text-xs">
          Select a group from the explorer to manage its settings, members, roles, and children.
        </p>
      </div>
    </div>
  )
}
