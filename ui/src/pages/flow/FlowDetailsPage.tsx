import { useEffect } from 'react'

import { History, Layout, Loader2, Settings } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useSetBreadcrumb } from '@/features/breadcrumb/model/useBreadcrumbStore'
import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { FlowDetailsOverviewTab } from '@/features/flow/components/FlowDetailsOverviewTab.tsx'
// Helper component for Overview to keep it clean (you can put this in a separate file too)
import { FlowDetailsSettingsTab } from '@/features/flow/components/FlowDetailsSettingsTab.tsx'
import { FlowHeader } from '@/features/flow/components/FlowHeader.tsx'
import { FlowHistoryTab } from '@/features/flow/components/FlowHistoryTab.tsx'
import { FlowTabLayout } from '@/features/flow/components/FlowTabLayout.tsx'

export function FlowDetailsPage() {
  const { flowId, tab } = useParams<{ flowId: string; tab?: string }>()
  const navigate = useRealmNavigate()

  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)

  useSetBreadcrumb({ [flowId ?? '']: draft?.name ?? '' })

  const validTabs = ['overview', 'history', 'settings']
  const activeTab = validTabs.includes(tab || '') ? (tab as string) : 'overview'

  const handleTabChange = (newTab: string) => flowId && navigate(`/flows/${flowId}/${newTab}`)

  useEffect(() => {
    if (!tab && flowId) navigate(`/flows/${flowId}/overview`, { replace: true })
  }, [tab, flowId, navigate])

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Flow...</p>
      </div>
    )
  }

  if (isError || !draft) {
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load flow details.</p>
        <Button variant="outline" onClick={() => navigate(-1)}>
          Go Back
        </Button>
      </div>
    )
  }

  return (
    <div className="bg-background flex h-full w-full flex-col">
      <FlowHeader draft={draft} />

      <Tabs
        value={activeTab}
        onValueChange={handleTabChange}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 border-b px-6 pt-2">
          <TabsList variant="line" className="gap-6 bg-transparent p-0">
            <TabsTrigger variant="line" value="overview" className="tab-trigger-styles">
              <Layout className="mr-2 h-4 w-4" /> Overview
            </TabsTrigger>
            <TabsTrigger variant="line" value="history" className="tab-trigger-styles">
              <History className="mr-2 h-4 w-4" /> Version History
            </TabsTrigger>
            <TabsTrigger variant="line" value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="overview" className="relative mt-0 h-full w-full flex-1">
          <FlowDetailsOverviewTab draft={draft} />
        </TabsContent>

        <TabsContent value="history" className="mt-0 flex-1 overflow-auto">
          <FlowHistoryTab flowId={draft.id} activeVersion={draft.active_version} />
        </TabsContent>

        <TabsContent
          value="settings"
          className="bg-muted/5 mt-0 min-h-0 w-full flex-1 overflow-y-auto p-6"
        >
          <FlowTabLayout draft={draft}>
            <FlowDetailsSettingsTab draft={draft} />
          </FlowTabLayout>
        </TabsContent>
      </Tabs>
    </div>
  )
}
