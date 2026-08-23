import { Layers, Loader2, Settings, ShieldCheck, Users } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useSetBreadcrumb } from '@/features/breadcrumb/model/useBreadcrumbStore'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useRoutedTab } from '@/entities/realm/lib/useRoutedTab'
import { useClient } from '@/features/client/api/useClient'
import { useRole } from '@/features/roles/api/useRole'
import { RoleHeader } from '@/features/roles/components/RoleHeader'
import { RoleCompositesTab } from '@/features/roles/components/RoleCompositesTab'
import { RoleMembersTab } from '@/features/roles/components/RoleMembersTab'
import { RolePermissionsTab } from '@/features/roles/components/RolePermissionsTab'
import { RoleSettingsTab } from '@/features/roles/components/RoleSettingsTab'
import { RoleTabLayout } from '@/features/roles/components/RoleTabLayout'

/** Tab slugs, in display order. The first is the default. */
const ROLE_TABS = ['settings', 'permissions', 'composites', 'members'] as const

export function EditRolePage() {
  const { roleId, clientId } = useParams<{
    roleId: string
    clientId?: string
  }>()
  const navigate = useRealmNavigate()

  const { data: role, isLoading, isError } = useRole(roleId!)
  // Only fetches when reached via the client-scoped path; resolves the client's
  // breadcrumb label (Clients > {client} > Roles > {role}).
  const { data: client } = useClient(clientId ?? '')

  // Base path is context-aware so tab switches / redirects stay nested when the
  // role was opened from a client.
  const basePath = clientId ? `/clients/${clientId}/roles/${roleId}` : `/roles/${roleId}`
  const { activeTab, onTabChange } = useRoutedTab({
    tabs: ROLE_TABS,
    basePath,
    enabled: Boolean(roleId),
  })
  const rolesListPath = clientId ? `/clients/${clientId}/roles` : '/roles'

  useSetBreadcrumb({
    [roleId ?? '']: role?.name ?? '',
    ...(clientId ? { [clientId]: client?.client_id ?? '' } : {}),
  })


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
        <Button variant="outline" onClick={() => navigate(rolesListPath)}>
          Go Back
        </Button>
      </div>
    )

  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden">
      <div className="shrink-0 px-6 pt-6">
        <RoleHeader role={role} />
      </div>

      <Tabs
        value={activeTab}
        onValueChange={onTabChange}
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

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 min-h-full w-full p-6">
            <RoleTabLayout role={role}>
              <RoleSettingsTab role={role} clientId={clientId} />
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
