import { type DragEvent, useCallback, useMemo, useRef } from 'react'

import { Background, Controls, ReactFlow, useReactFlow } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider.tsx'
import { AuthenticatorNode } from '@/features/flow-builder/components/nodes/AuthenticatorNode.tsx'
import { StartNode } from '@/features/flow-builder/components/nodes/StartNode.tsx'
import { TerminalNode } from '@/features/flow-builder/components/nodes/TerminalNode.tsx'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore.ts'

export function FlowCanvas() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null)
  const { nodes, edges, onNodesChange, onEdgesChange, onConnect, addNode, selectNode } =
    useFlowBuilderStore()

  const { screenToFlowPosition } = useReactFlow()
  const { theme } = useTheme()
  const isDark = theme === 'dark'

  const nodeTypes = useMemo(
    () => ({
      // Start
      'core.start': StartNode,

      // Authenticators
      'core.auth.password': AuthenticatorNode,
      'core.auth.otp': AuthenticatorNode,
      'core.auth.registration': AuthenticatorNode,

      // Logic
      'core.logic.condition': AuthenticatorNode, // Or a specific LogicNode
      'core.logic.script': AuthenticatorNode,

      // Terminals
      'core.terminal.allow': TerminalNode, // Or AuthenticatorNode if you reuse it
      'core.terminal.deny': TerminalNode,

      // Fallback for drag-and-drop category matching if specific ID fails
      authenticator: AuthenticatorNode,
      terminal: TerminalNode,
    }),
    [],
  )

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()

      // 1. Get Data from Palette
      const droppedId = event.dataTransfer.getData('application/reactflow/type') // "core.auth.password"
      const droppedCategory = event.dataTransfer.getData('application/reactflow/category')
      const droppedOutputsStr = event.dataTransfer.getData('application/reactflow/outputs') // Need this!

      if (!droppedId) return

      // Parse outputs (handles) if passed from palette
      let outputs = []
      try {
        outputs = droppedOutputsStr ? JSON.parse(droppedOutputsStr) : []
      } catch (e) {
        console.warn('Failed to parse outputs', e)
      }

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      })

      // 2. Create Node
      const newNode = {
        id: crypto.randomUUID(),
        // CRITICAL: Type must match the Backend ID ("core.auth.password")
        type: droppedId,
        position,
        data: {
          label: droppedId, // Or a display name if passed
          config: {},
          category: droppedCategory,
          // CRITICAL: Pass outputs to data so the Node Component can render handles
          outputs: outputs,
        },
      }

      addNode(newNode)
    },
    [screenToFlowPosition, addNode],
  )

  const proOptions = { hideAttribution: true }

  return (
    <div className="h-full w-full flex-1" ref={reactFlowWrapper}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={(_, node) => selectNode(node.id)}
        onPaneClick={() => selectNode(null)}
        onDrop={onDrop}
        onDragOver={onDragOver}
        fitView
        nodeTypes={nodeTypes}
        proOptions={proOptions}
        colorMode={isDark ? 'dark' : 'light'}
      >
        <Background color={isDark ? '#333' : '#aaa'} gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  )
}
