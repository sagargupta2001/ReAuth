import { useEffect } from 'react'

import { ArrowLeft, Layers, Loader2, Settings, ShieldCheck, Users } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button, buttonVariants } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useRole } from '@/features/roles/api/useRole'
import { RoleHeader } from '@/features/roles/components/RoleHeader'
import { RoleCompositesTab } from '@/features/roles/components/RoleCompositesTab'
import { RoleMembersTab } from '@/features/roles/components/RoleMembersTab'
import { RolePermissionsTab } from '@/features/roles/components/RolePermissionsTab'
import { RoleSettingsTab } from '@/features/roles/components/RoleSettingsTab'
import { RoleTabLayout } from '@/features/roles/components/RoleTabLayout'
import { cn } from '@/lib/utils'

export function EditRolePage() {
  const { roleId, tab } = useParams<{ roleId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const { data: role, isLoading, isError } = useRole(roleId!)

  const validTabs = ['settings', 'permissions', 'composites', 'members']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'settings'

  useEffect(() => {
    if (!tab) {
      navigate(`/roles/${roleId}/settings`, { replace: true })
    }
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
      <div className="mb-2 shrink-0">
        <RealmLink
          to="/roles"
          className={cn(
            buttonVariants({ variant: 'link', size: 'sm' }),
            'text-muted-foreground hover:text-foreground gap-2 pl-0',
          )}
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Roles
        </RealmLink>
      </div>

      <RoleHeader role={role} />

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
          <TabsList variant="line" className="gap-6 bg-transparent p-0">
            <TabsTrigger variant="line" value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>

            <TabsTrigger variant="line" value="permissions" className="tab-trigger-styles">
              <ShieldCheck className="mr-2 h-4 w-4" /> Permissions
            </TabsTrigger>

            <TabsTrigger variant="line" value="composites" className="tab-trigger-styles">
              <Layers className="mr-2 h-4 w-4" /> Composites
            </TabsTrigger>

            <TabsTrigger variant="line" value="members" className="tab-trigger-styles">
              <Users className="mr-2 h-4 w-4" /> Members
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto xl:overflow-hidden">
          <TabsContent value="settings" className="mt-0 h-full w-full p-6">
            <RoleTabLayout role={role}>
              <RoleSettingsTab role={role} />
            </RoleTabLayout>
          </TabsContent>

          <TabsContent value="permissions" className="mt-0 h-full w-full">
            <RolePermissionsTab roleId={role.id} clientId={role.client_id ?? null} />
          </TabsContent>

          <TabsContent value="composites" className="mt-0 h-full w-full p-6">
            <RoleCompositesTab roleId={role.id} />
          </TabsContent>

          <TabsContent value="members" className="mt-0 h-full w-full p-6">
            <RoleMembersTab roleId={role.id} />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
