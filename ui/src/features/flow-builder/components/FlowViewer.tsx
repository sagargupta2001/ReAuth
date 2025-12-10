import { ReactFlow, ReactFlowProvider } from '@xyflow/react'
import type { Edge, Node } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider'
import { AuthenticatorNode } from '@/features/flow-builder/components/nodes/AuthenticatorNode'

// Reuse your existing node types so they look identical to the builder
const nodeTypes = {
  authenticator: AuthenticatorNode,
}

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
        nodeTypes={nodeTypes}
        fitView
        // --- READ ONLY SETTINGS ---
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        panOnDrag={true} // Allow users to pan around
        zoomOnScroll={true} // Allow users to zoom
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
