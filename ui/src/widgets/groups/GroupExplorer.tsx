import { useEffect, useMemo, useState } from 'react'

import { Loader2, Plus } from 'lucide-react'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useGroup } from '@/features/group/api/useGroup'
import { CreateGroupForm } from '@/features/group/forms/CreateGroupForm'
import { GroupChildrenTab } from '@/features/group/components/GroupChildrenTab'
import { GroupHeader } from '@/features/group/components/GroupHeader'
import { GroupMembersTab } from '@/features/group/components/GroupMembersTab'
import { GroupRolesTab } from '@/features/group/components/GroupRolesTab'
import { GroupSettingsTab } from '@/features/group/components/GroupSettingsTab'
import { GroupTreePanel } from '@/features/group-tree/components/GroupTreePanel'

interface GroupExplorerProps {
  groupId?: string
  tab?: string
}

const validTabs = ['settings', 'members', 'roles', 'children']

export function GroupExplorer({ groupId, tab }: GroupExplorerProps) {
  const navigate = useRealmNavigate()
  const [createOpen, setCreateOpen] = useState(false)
  const [createParentId, setCreateParentId] = useState<string | null>(null)
  const [refreshKey, setRefreshKey] = useState(0)

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

  const handleSelectGroup = (id: string) => {
    navigate(`/groups/${id}/settings`)
  }

  const handleCreateGroup = (parentId: string | null) => {
    setCreateParentId(parentId)
    setCreateOpen(true)
  }

  const handleCreateClose = () => {
    setCreateOpen(false)
    setCreateParentId(null)
  }

  return (
    <div className="flex h-full flex-1 flex-col gap-4">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Groups</h2>
          <p className="text-muted-foreground">
            Organize users and roles into hierarchical groups.
          </p>
        </div>
        <Button onClick={() => handleCreateGroup(null)} className="space-x-1">
          <span>Create Group</span> <Plus size={18} />
        </Button>
      </div>

      <div className="flex overflow-hidden rounded-xl border bg-muted/5 h-[calc(100vh-225px)]">
        <div className="w-[320px] shrink-0 border-r bg-background/60">
          <GroupTreePanel
            selectedId={groupId}
            onSelect={handleSelectGroup}
            onCreateGroup={handleCreateGroup}
            refreshKey={refreshKey}
          />
        </div>

        <div className="flex flex-1 flex-col overflow-hidden">
          {!groupId ? (
            <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-2">
              <p>Select a group to manage its members and roles.</p>
              <Button variant="outline" onClick={() => handleCreateGroup(null)}>
                Create a group
              </Button>
            </div>
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
                  <TabsList className="gap-6 bg-transparent p-0">
                    <TabsTrigger value="settings" className="tab-trigger-styles">
                      Settings
                    </TabsTrigger>
                    <TabsTrigger value="members" className="tab-trigger-styles">
                      Members
                    </TabsTrigger>
                    <TabsTrigger value="roles" className="tab-trigger-styles">
                      Roles
                    </TabsTrigger>
                    <TabsTrigger value="children" className="tab-trigger-styles">
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
      </div>

      <Dialog
        open={createOpen}
        onOpenChange={(open) => {
          if (!open) {
            handleCreateClose()
          } else {
            setCreateOpen(true)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{createParentId ? 'Create Sub-group' : 'Create Group'}</DialogTitle>
            <DialogDescription>
              {createParentId
                ? 'Add a new group under the selected parent.'
                : 'Create a new top-level group in this realm.'}
            </DialogDescription>
          </DialogHeader>

          <CreateGroupForm
            isDialog
            parentId={createParentId}
            onSuccess={() => {
              setRefreshKey((prev) => prev + 1)
              handleCreateClose()
            }}
            onCancel={handleCreateClose}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}
