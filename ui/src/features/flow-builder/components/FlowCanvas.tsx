import { type DragEvent, useCallback, useRef } from 'react'

import { Background, Controls, ReactFlow, useReactFlow } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/themeProvider.tsx'
import { AuthenticatorNode } from '@/features/flow-builder/components/nodes/AuthenticatorNode.tsx'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore.ts'

const nodeTypes = {
  authenticator: AuthenticatorNode,
  // we will add 'logic' and 'terminal' later
}

export function FlowCanvas() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null)
  const { nodes, edges, onNodesChange, onEdgesChange, onConnect, addNode, selectNode } =
    useFlowBuilderStore()

  const { screenToFlowPosition } = useReactFlow()
  const { theme } = useTheme()
  const isDark = theme === 'dark'

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault()

      // Read the correct keys set in NodePalette
      const droppedId = event.dataTransfer.getData('application/reactflow/type') // e.g., "authenticator.password"
      const droppedCategory = event.dataTransfer.getData('application/reactflow/category') // e.g., "authenticator"

      // If we don't have an ID, we can't create a node
      if (!droppedId) return

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      })

      // Use the category to determine the React Flow component type
      let nodeComponentType = 'default'

      if (droppedCategory === 'Authenticator') {
        nodeComponentType = 'authenticator'
      } else if (droppedCategory === 'Terminal') {
        // This is CRITICAL. The backend validator looks for exactly "terminal"
        nodeComponentType = 'terminal'
      } else if (droppedCategory === 'Logic') {
        nodeComponentType = 'default' // Or 'logic' if you have that component
      }

      const newNode = {
        id: crypto.randomUUID(),
        type: nodeComponentType,
        position,
        data: {
          label: droppedId, // Store the specific ID (e.g. "authenticator.password") for logic later
          config: {},
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
