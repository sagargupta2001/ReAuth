import { useMemo } from 'react'

import { Edit, Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.tsx'
import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { FlowViewer } from '@/features/flow-builder/components/FlowViewer'

export function FlowDetailsPage() {
  const { flowId } = useParams()
  const navigate = useRealmNavigate()

  // Reuse the existing hook to fetch the flow data
  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)

  // Parse the graph JSON safely
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
        <p>Loading Flow Details...</p>
      </div>
    )
  }

  if (isError || !draft) {
    return (
      <div className="text-destructive flex h-full w-full flex-col items-center justify-center gap-2">
        <p>Failed to load flow.</p>
        <Button variant="outline" onClick={() => navigate(-1)}>
          Go Back
        </Button>
      </div>
    )
  }

  return (
    <div className="bg-background flex h-full w-full flex-col">
      <header className="flex h-20 shrink-0 items-center justify-between border-b px-6 py-4">
        <div className="flex items-start gap-4">
          <div className="flex flex-col gap-1">
            <h1 className="text-foreground text-xl font-bold tracking-tight">{draft.name}</h1>
            <p className="text-muted-foreground line-clamp-1 max-w-lg text-sm">
              {draft.description || 'No description provided.'}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <Button onClick={() => navigate(`/flows/${flowId}/builder`)} className="gap-2">
            <Edit className="h-4 w-4" />
            Open in Editor
          </Button>
        </div>
      </header>

      <main className="bg-muted/5 flex-1 overflow-hidden">
        <FlowViewer nodes={nodes} edges={edges} />
      </main>
    </div>
  )
}
