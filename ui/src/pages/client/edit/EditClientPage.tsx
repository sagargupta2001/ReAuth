import { Loader2, Settings, Shield } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useRoutedTab } from '@/entities/realm/lib/useRoutedTab'
import { useSetBreadcrumb } from '@/features/breadcrumb/model/useBreadcrumbStore'
import { useClient } from '@/features/client/api/useClient'
import { ClientHeader } from '@/features/client/components/ClientHeader.tsx'
import { ClientRolesTab } from '@/features/client/components/ClientRolesTab.tsx'
import { ClientSettingsTab } from '@/features/client/components/ClientSettingsTab.tsx'
import { ClientTabLayout } from '@/features/client/components/ClientTabLayout.tsx'

/** Tab slugs, in display order. The first is the default. */
const CLIENT_TABS = ['settings', 'roles', 'advanced'] as const

export function EditClientPage() {
  const { clientId } = useParams<{ clientId: string }>()
  const navigate = useRealmNavigate()
  const { activeTab, onTabChange } = useRoutedTab({
    tabs: CLIENT_TABS,
    basePath: `/clients/${clientId}`,
    enabled: Boolean(clientId),
  })

  const { data: client, isLoading, isError } = useClient(clientId!)

  useSetBreadcrumb({ [clientId ?? '']: client?.client_id ?? '' })

  if (isLoading)
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Client...</p>
      </div>
    )


  if (isError || !client)
    return (
      <div className="flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load client details.</p>
        <Button variant="outline" onClick={() => navigate('/clients')}>
          Go Back
        </Button>
      </div>
    )


  return (
    <div className="bg-background flex h-full min-h-0 w-full min-w-0 flex-1 flex-col overflow-hidden p-6">
      <div className="shrink-0">
        <ClientHeader client={client} />
      </div>

      <Tabs
        value={activeTab}
        onValueChange={onTabChange}
        className="flex min-h-0 flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 shrink-0 border-b px-6 pt-2">
          <TabsList variant='line' className="gap-6 bg-transparent p-0">
            <TabsTrigger variant='line' value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>

            <TabsTrigger variant='line' value="roles" className="tab-trigger-styles">
              <Shield className="mr-2 h-4 w-4" /> Roles
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 min-h-0 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 min-h-full w-full p-6">
            <ClientTabLayout client={client}>
              <ClientSettingsTab client={client} />
            </ClientTabLayout>
          </TabsContent>

          <TabsContent value="roles" className="mt-0 h-full w-full">
            <ClientRolesTab clientId={client.id} />
          </TabsContent>

        </div>
      </Tabs>
    </div>
  )
}
