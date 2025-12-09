import { useCallback, useRef } from 'react'

import { Background, Controls, ReactFlow, ReactFlowProvider, useReactFlow } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider.tsx'
import { AuthenticatorNode } from '@/features/flow-builder/components/nodes/AuthenticatorNode.tsx'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore.ts'

const nodeTypes = {
  authenticator: AuthenticatorNode,
  // we will add 'logic' and 'terminal' later
}

function FlowCanvasInternal() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null)
  const { nodes, edges, onNodesChange, onEdgesChange, onConnect, addNode, selectNode } =
    useFlowBuilderStore()
  const { screenToFlowPosition } = useReactFlow()
  const { theme } = useTheme()
  const isDark = theme === 'dark'

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault()

      const droppedType = event.dataTransfer.getData('application/reactflow')
      if (!droppedType) return

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      })

      let nodeComponentType = 'default'
      if (droppedType.startsWith('authenticator.')) {
        nodeComponentType = 'authenticator'
      } else if (droppedType.startsWith('logic.')) {
        nodeComponentType = 'default' // Change to 'logic' when you have the component
      }

      const newNode = {
        id: crypto.randomUUID(),
        type: nodeComponentType, // Use the mapped type
        position,
        data: {
          label: droppedType, // e.g. "authenticator.password"
          config: {}, // Empty config to start
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
        <Background color="#aaa" gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  )
}

export function FlowCanvas() {
  return (
    <ReactFlowProvider>
      <FlowCanvasInternal />
    </ReactFlowProvider>
  )
}
