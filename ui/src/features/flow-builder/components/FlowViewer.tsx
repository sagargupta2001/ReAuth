import { ReactFlow, ReactFlowProvider } from '@xyflow/react'
import type { Edge, Node } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider'
import { flowNodeTypes } from '@/entities/flow/config/nodeTypes.ts'

interface FlowViewerProps {
  nodes: Node[]
  edges: Edge[]
}

function FlowViewerInternal({ nodes, edges }: FlowViewerProps) {
  const { theme } = useTheme()
  const isDark = theme === 'dark'

  return (
    <div className="h-full w-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={flowNodeTypes}
        fitView
        // --- READ ONLY SETTINGS ---
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        nodesFocusable={false}
        edgesFocusable={false}
        panOnDrag={true} // Allow panning
        zoomOnScroll={true} // Allow zooming
        zoomOnDoubleClick={false}
        proOptions={{ hideAttribution: true }}
        colorMode={isDark ? 'dark' : 'light'}
      />
    </div>
  )
}

export function FlowViewer(props: FlowViewerProps) {
  return (
    <ReactFlowProvider>
      <FlowViewerInternal {...props} />
    </ReactFlowProvider>
  )
}
