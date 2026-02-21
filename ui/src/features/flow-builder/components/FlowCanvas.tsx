import { type DragEvent, useCallback, useRef } from 'react'

import { Background, Controls, ReactFlow, useReactFlow } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useTheme } from '@/app/providers/ThemeContext'
import { flowNodeTypes } from '@/entities/flow/config/nodeTypes.ts'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore.ts'

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

      // 1. Get Data from Palette
      const droppedId = event.dataTransfer.getData('application/reactflow/type')
      const droppedCategory = event.dataTransfer.getData('application/reactflow/category')
      const droppedOutputsStr = event.dataTransfer.getData('application/reactflow/outputs')

      if (!droppedId) return

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
        // Matches the backend registry key (e.g. "core.auth.password")
        type: droppedId,
        position,
        data: {
          label: droppedId,
          config: {},
          category: droppedCategory,
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
        nodeTypes={flowNodeTypes}
        proOptions={proOptions}
        colorMode={isDark ? 'dark' : 'light'}
      >
        <Background color={isDark ? '#333' : '#aaa'} gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  )
}
