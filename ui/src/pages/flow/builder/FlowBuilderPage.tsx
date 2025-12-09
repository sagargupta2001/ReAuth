import { useEffect } from 'react'

import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { BuilderHeader } from '@/features/flow-builder/components/BuilderHeader'
import { FlowCanvas } from '@/features/flow-builder/components/FlowCanvas'
import { NodeInspector } from '@/features/flow-builder/components/NodeInspector'
import { NodePalette } from '@/features/flow-builder/components/NodePalette'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'

export function FlowBuilderPage() {
  const { flowId } = useParams()
  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)
  const setGraph = useFlowBuilderStore((s) => s.setGraph)

  // Sync DB -> Store
  useEffect(() => {
    if (draft?.graph_json) {
      // React Flow expects { nodes: [], edges: [] }
      // If empty JSON {}, default to arrays
      const { nodes = [], edges = [] } = draft.graph_json
      setGraph(nodes, edges)
    }
  }, [draft, setGraph])

  if (isLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Flow Draft...</p>
      </div>
    )
  }

  if (isError)
    return (
      <div className="text-destructive flex h-full w-full items-center justify-center">
        Failed to load flow. Please try again.
      </div>
    )

  return (
    <div className="flex h-full w-full flex-col">
      <BuilderHeader flowName={draft?.name || 'Untitled Flow'} flowId={flowId!} />

      <div className="relative flex flex-1 overflow-hidden">
        <NodePalette />

        <div className="relative h-full flex-1">
          <FlowCanvas />
        </div>

        <NodeInspector />
      </div>
    </div>
  )
}
