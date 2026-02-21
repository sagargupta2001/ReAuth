import { useMemo } from 'react'

import { type Edge, type Node } from '@xyflow/react'

import type { FlowDraft } from '@/entities/flow/model/types.ts'
import { FlowViewer } from '@/features/flow-builder/components/FlowViewer.tsx'

export function FlowDetailsOverviewTab({ draft }: { draft: FlowDraft }) {
  const { nodes, edges } = useMemo(() => {
    const graph = draft?.graph_json as { nodes?: Node[]; edges?: Edge[] } | undefined
    if (!graph) return { nodes: [], edges: [] }
    return {
      nodes: graph.nodes || [],
      edges: graph.edges || [],
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
