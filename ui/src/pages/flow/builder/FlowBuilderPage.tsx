import { useEffect } from 'react'

import { ReactFlowProvider } from '@xyflow/react'
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
  // Ensure we have a string, though the router guarantees this param exists
  const draftId = flowId!

  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)
  const { setGraph, reset } = useFlowBuilderStore()

  // Sync DB -> Store
  useEffect(() => {
    if (draft?.graph_json) {
      // React Flow expects { nodes: [], edges: [] }
      // If empty JSON {}, default to arrays
      const { nodes = [], edges = [] } = draft.graph_json
      setGraph(nodes, edges)
    }
  }, [draft, setGraph])

  // Cleanup on Unmount (Prevent old graph from flashing when opening a new one)
  useEffect(() => {
    return () => {
      reset()
    }
  }, [reset])

  if (isLoading)
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Flow Draft...</p>
      </div>
    )

  if (isError)
    return (
      <div className="text-destructive flex h-full w-full items-center justify-center">
        Failed to load flow. Please try again.
      </div>
    )

  return (
    <ReactFlowProvider>
      <div className="flex h-full w-full flex-col">
        <BuilderHeader
          flowName={draft?.name || 'Untitled Flow'}
          flowId={draftId}
          activeVersion={draft?.active_version_number}
        />

        <div className="relative flex flex-1 overflow-hidden">
          <NodePalette />

          <div className="relative h-full flex-1">
            <FlowCanvas />
          </div>

          <NodeInspector />
        </div>
      </div>
    </ReactFlowProvider>
  )
}
