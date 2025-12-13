import { ReactFlow, ReactFlowProvider } from '@xyflow/react'
import type { Edge, Node } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider'
import { AuthenticatorNode } from '@/features/flow-builder/components/nodes/AuthenticatorNode'
import { StartNode } from '@/features/flow-builder/components/nodes/StartNode'
import { TerminalNode } from '@/features/flow-builder/components/nodes/TerminalNode'

// 1. Updated Node Types Map
// This must match FlowCanvas exactly so the "View" mode looks just like "Edit" mode.
const nodeTypes = {
  // Start
  'core.start': StartNode,

  // Authenticators
  'core.auth.password': AuthenticatorNode,
  'core.auth.otp': AuthenticatorNode,
  'core.auth.registration': AuthenticatorNode,

  // Logic (Using AuthenticatorNode as placeholder for now, or use a dedicated LogicNode if you have one)
  'core.logic.condition': AuthenticatorNode,
  'core.logic.script': AuthenticatorNode,

  // Terminals
  'core.terminal.allow': TerminalNode,
  'core.terminal.deny': TerminalNode,

  // Fallbacks (legacy support)
  authenticator: AuthenticatorNode,
  terminal: TerminalNode,
  start: StartNode,
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
