import { useMemo, useState } from 'react'

import { History, Layout, Loader2, Settings } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
// Helper component for Overview to keep it clean (you can put this in a separate file too)
import { FlowViewer } from '@/features/flow-builder/components/FlowViewer'
import { FlowHeader } from '@/features/flow/components/FlowHeader.tsx'
import { FlowHistoryTab } from '@/features/flow/components/FlowHistoryTab.tsx'

export function FlowDetailsPage() {
  const { flowId } = useParams()
  const navigate = useRealmNavigate()
  const [activeTab, setActiveTab] = useState('overview')

  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)

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
        onValueChange={setActiveTab}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 border-b px-6 pt-2">
          <TabsList className="gap-6 bg-transparent p-0">
            <TabsTrigger value="overview" className="tab-trigger-styles">
              <Layout className="mr-2 h-4 w-4" /> Overview
            </TabsTrigger>
            <TabsTrigger value="history" className="tab-trigger-styles">
              <History className="mr-2 h-4 w-4" /> Version History
            </TabsTrigger>
            <TabsTrigger value="settings" className="tab-trigger-styles">
              <Settings className="mr-2 h-4 w-4" /> Settings
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="overview" className="relative mt-0 h-full w-full flex-1">
          <FlowOverviewTab draft={draft} />
        </TabsContent>

        <TabsContent value="history" className="mt-0 flex-1 overflow-auto">
          <FlowHistoryTab flowId={draft.id} activeVersion={draft.active_version} />
        </TabsContent>

        <TabsContent value="settings" className="mt-0 flex-1">
          <FlowSettingsTab />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function FlowOverviewTab({ draft }: { draft: any }) {
  const { nodes, edges } = useMemo(() => {
    if (!draft?.graph_json) return { nodes: [], edges: [] }
    return {
      nodes: draft.graph_json.nodes || [],
      edges: draft.graph_json.edges || [],
    }
  }, [draft])

  return (
    <>
      <div className="bg-background/80 pointer-events-none absolute top-4 left-4 z-10 max-w-sm rounded-md border p-3 shadow-sm backdrop-blur">
        <h3 className="text-muted-foreground mb-1 text-xs font-semibold uppercase">Description</h3>
        <p className="text-sm">{draft.description || 'No description configured.'}</p>
      </div>
      <div className="bg-muted/5 h-full w-full">
        <FlowViewer nodes={nodes} edges={edges} />
      </div>
    </>
  )
}

// Helper placeholder for Settings
function FlowSettingsTab() {
  return (
    <div className="max-w-2xl space-y-6 p-6">
      <div className="rounded-md border p-4">
        <h3 className="mb-1 font-medium">General Settings</h3>
        <p className="text-muted-foreground mb-4 text-sm">
          Manage the basic information of this flow.
        </p>
        <Button variant="outline" disabled>
          Save Changes
        </Button>
      </div>
    </div>
  )
}
