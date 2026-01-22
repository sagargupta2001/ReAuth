import { useEffect } from 'react'

import { Activity, Loader2, Settings, Shield } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useClient } from '@/features/client/api/useClient'
import { ClientHeader } from '@/features/client/components/ClientHeader.tsx'
import { ClientRolesTab } from '@/features/client/components/ClientRolesTab.tsx'
import { ClientSettingsTab } from '@/features/client/components/ClientSettingsTab.tsx'

export function EditClientPage() {
  const { clientId, tab } = useParams<{ clientId: string; tab?: string }>()
  const navigate = useRealmNavigate()
  const validTabs = ['settings', 'roles', 'advanced']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'settings'

  const { data: client, isLoading, isError } = useClient(clientId!)

  const handleTabChange = (newTab: string) =>
    navigate(`/clients/${clientId}/${newTab}`)


  useEffect(() => {
    !tab && navigate(`/clients/${clientId}/settings`, { replace: true })
  }, [tab, clientId, navigate])

  if (isLoading)
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Client...</p>
      </div>
    )


  if (isError || !client)
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load client details.</p>
        <Button variant="outline" onClick={() => navigate('/clients')}>
          Go Back
        </Button>
      </div>
    )


  return (
    <div className="bg-background flex h-full w-full flex-col overflow-hidden p-12">
      <div className="shrink-0">
        <ClientHeader client={client} />
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

            <TabsTrigger value="roles" className="tab-trigger-styles">
              <Shield className="mr-2 h-4 w-4" /> Roles
            </TabsTrigger>

            <TabsTrigger value="advanced" className="tab-trigger-styles">
              <Activity className="mr-2 h-4 w-4" /> Advanced
            </TabsTrigger>
          </TabsList>
        </div>

        <div className="bg-muted/5 flex-1 overflow-y-auto">
          <TabsContent value="settings" className="mt-0 h-full w-full">
            <ClientSettingsTab client={client} />
          </TabsContent>

          <TabsContent value="roles" className="mt-0 h-full w-full">
            <ClientRolesTab clientId={client.id} />
          </TabsContent>

          <TabsContent value="advanced" className="mt-0 h-full w-full p-6">
            <div className="text-muted-foreground flex h-64 flex-col items-center justify-center rounded-lg border-2 border-dashed">
              <p>Advanced OAuth2 settings coming soon.</p>
            </div>
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
