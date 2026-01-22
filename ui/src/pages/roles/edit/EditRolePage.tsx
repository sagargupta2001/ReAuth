import { useEffect } from 'react'

import { Loader2, Settings, ShieldCheck, Users } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useRole } from '@/features/roles/api/useRole'
import { RoleHeader } from '@/features/roles/components/RoleHeader'
import { RolePermissionsTab } from '@/features/roles/components/RolePermissionsTab'
import { RoleSettingsTab } from '@/features/roles/components/RoleSettingsTab'

export function EditRolePage() {
  const { roleId, tab } = useParams<{ roleId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const { data: role, isLoading, isError } = useRole(roleId!)

  const validTabs = ['settings', 'permissions', 'members']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'settings'

  useEffect(() => {
    !tab && navigate(`/roles/${roleId}/settings`, { replace: true })
  }, [tab, roleId, navigate])

  const handleTabChange = (newTab: string) =>
    navigate(`/roles/${roleId}/${newTab}`)


  if (isLoading)
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Role...</p>
      </div>
    )


  if (isError || !role)
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Role not found.</p>
        <Button variant="outline" onClick={() => navigate('/roles')}>
          Go Back
        </Button>
      </div>
    )

  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden p-6">
      <div className="shrink-0">
        <RoleHeader role={role} />
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

            <TabsTrigger value="permissions" className="tab-trigger-styles">
              <ShieldCheck className="mr-2 h-4 w-4" /> Permissions
            </TabsTrigger>

            <TabsTrigger value="members" className="tab-trigger-styles">
              <Users className="mr-2 h-4 w-4" /> Members
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 h-full w-full">
            <RoleSettingsTab role={role} />
          </TabsContent>

          <TabsContent value="permissions" className="mt-0 h-full w-full">
            <RolePermissionsTab roleId={role.id} />
          </TabsContent>

          <TabsContent value="members" className="mt-0 h-full w-full p-6">
            <div className="text-muted-foreground flex h-64 flex-col items-center justify-center rounded-lg border-2 border-dashed">
              <Users className="mb-2 h-8 w-8 opacity-50" />
              <p>User assignment UI coming soon.</p>
            </div>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
