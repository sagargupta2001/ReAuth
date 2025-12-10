import { useMemo, useState } from 'react'

import {
  GitBranch,
  History,
  Layout,
  Loader2,
  Lock,
  MoreVertical,
  Pencil,
  Settings,
  ShieldCheck,
} from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Badge } from '@/components/badge'
import { Button } from '@/components/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/tabs'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { FlowViewer } from '@/features/flow-builder/components/FlowViewer'

export function FlowDetailsPage() {
  const { flowId } = useParams()
  const navigate = useRealmNavigate()

  // We default to the 'overview' tab
  const [activeTab, setActiveTab] = useState('overview')

  // Fetch the Draft (now includes active_version and built_in from backend)
  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)

  // Parse the graph JSON safely for the viewer
  const { nodes, edges } = useMemo(() => {
    if (!draft?.graph_json) return { nodes: [], edges: [] }
    return {
      nodes: draft.graph_json.nodes || [],
      edges: draft.graph_json.edges || [],
    }
  }, [draft])

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

  const isSystemFlow = draft.built_in

  return (
    <div className="bg-background flex h-full w-full flex-col">
      {/* --- 1. CONTROL CENTER HEADER --- */}
      <header className="flex h-16 shrink-0 items-center justify-between border-b px-6">
        <div className="flex items-center gap-4">
          <div className="bg-primary/10 flex h-10 w-10 items-center justify-center rounded-lg">
            {isSystemFlow ? (
              <Lock className="text-primary h-5 w-5" />
            ) : (
              <GitBranch className="text-primary h-5 w-5" />
            )}
          </div>

          <div className="flex flex-col">
            <div className="flex items-center gap-2">
              <h1 className="text-foreground text-lg font-bold tracking-tight">{draft.name}</h1>
              {isSystemFlow ? (
                <Badge variant="secondary" className="h-5 text-[10px]">
                  System
                </Badge>
              ) : (
                <Badge variant="outline" className="h-5 text-[10px]">
                  Custom
                </Badge>
              )}
            </div>
            <span className="text-muted-foreground text-xs">
              ID: <span className="font-mono opacity-70">{draft.id.slice(0, 8)}...</span>
            </span>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {/* ✅ FIX: Dynamic Status Indicator */}
          <div className="text-muted-foreground mr-2 flex items-center gap-2 border-r px-3 text-xs">
            {draft.active_version ? (
              <>
                <span className="relative flex h-2 w-2">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"></span>
                  <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500"></span>
                </span>
                Active Version:{' '}
                <span className="text-foreground font-semibold">v{draft.active_version}</span>
              </>
            ) : (
              <>
                <span className="relative flex h-2 w-2">
                  <span className="relative inline-flex h-2 w-2 rounded-full bg-yellow-500"></span>
                </span>
                Status: <span className="text-foreground font-semibold">Unpublished Draft</span>
              </>
            )}
          </div>

          <Button onClick={() => navigate(`/flows/${flowId}/builder`)} className="gap-2">
            <Pencil className="h-3.5 w-3.5" />
            {isSystemFlow ? 'Edit Flow' : 'Edit Draft'}
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon">
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem>Duplicate</DropdownMenuItem>
              <DropdownMenuItem className="text-destructive">Delete</DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      {/* --- 2. TABS NAVIGATION --- */}
      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="flex flex-1 flex-col overflow-hidden"
      >
        <div className="bg-muted/5 border-b px-6 pt-2">
          <TabsList className="gap-6 bg-transparent p-0">
            <TabsTrigger
              value="overview"
              className="data-[state=active]:border-primary text-muted-foreground data-[state=active]:text-foreground rounded-none px-0 pt-2 pb-3 data-[state=active]:border-b-2 data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <Layout className="mr-2 h-4 w-4" />
              Overview
            </TabsTrigger>
            <TabsTrigger
              value="history"
              className="data-[state=active]:border-primary text-muted-foreground data-[state=active]:text-foreground rounded-none px-0 pt-2 pb-3 data-[state=active]:border-b-2 data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <History className="mr-2 h-4 w-4" />
              Version History
            </TabsTrigger>
            <TabsTrigger
              value="settings"
              className="data-[state=active]:border-primary text-muted-foreground data-[state=active]:text-foreground rounded-none px-0 pt-2 pb-3 data-[state=active]:border-b-2 data-[state=active]:bg-transparent data-[state=active]:shadow-none"
            >
              <Settings className="mr-2 h-4 w-4" />
              Settings
            </TabsTrigger>
          </TabsList>
        </div>

        {/* --- 3. TAB CONTENT --- */}

        {/* VISUALIZATION / OVERVIEW */}
        <TabsContent value="overview" className="relative mt-0 h-full w-full flex-1">
          {/* Overlay description for context */}
          <div className="bg-background/80 pointer-events-none absolute top-4 left-4 z-10 max-w-sm rounded-md border p-3 shadow-sm backdrop-blur">
            <h3 className="text-muted-foreground mb-1 text-xs font-semibold uppercase">
              Description
            </h3>
            <p className="text-sm">{draft.description || 'No description configured.'}</p>
          </div>

          <div className="bg-muted/5 h-full w-full">
            <FlowViewer nodes={nodes} edges={edges} />
          </div>
        </TabsContent>

        {/* HISTORY (Placeholder for Milestone 3) */}
        <TabsContent value="history" className="mt-0 flex-1 overflow-auto p-6">
          <div className="bg-card rounded-md border">
            <div className="bg-muted/30 border-b p-4">
              <h3 className="font-semibold">Deployment History</h3>
              <p className="text-muted-foreground text-sm">
                View and rollback to previous versions of this flow.
              </p>
            </div>
            <div className="text-muted-foreground flex flex-col items-center justify-center gap-2 p-12 text-center">
              <ShieldCheck className="h-10 w-10 opacity-20" />
              {draft.active_version ? (
                // If we have an active version, we likely have history
                // For now, placeholder text, but logic ready for table
                <p>Latest version: v{draft.active_version}</p>
              ) : (
                <p>No version history available yet.</p>
              )}
              <p className="text-xs">Publish your first draft to see versions here.</p>
            </div>
          </div>
        </TabsContent>

        {/* SETTINGS */}
        <TabsContent value="settings" className="mt-0 flex-1 p-6">
          <div className="max-w-2xl space-y-6">
            <div className="rounded-md border p-4">
              <h3 className="mb-1 font-medium">General Settings</h3>
              <p className="text-muted-foreground mb-4 text-sm">
                Manage the basic information of this flow.
              </p>
              {/* Forms would go here */}
              <Button variant="outline" disabled>
                Save Changes
              </Button>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
