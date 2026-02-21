import {
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  type OnConnect,
  type OnEdgesChange,
  type OnNodesChange,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
} from '@xyflow/react'
import { create } from 'zustand'

export interface NodeMetadata {
  id: string
  category: string
  display_name: string
  description: string
  icon: string
  inputs: string[]
  outputs: string[]
  config_schema: Record<string, unknown>
}

interface FlowBuilderState {
  nodes: Node[]
  edges: Edge[]
  selectedNodeId: string | null
  nodeTypes: NodeMetadata[]
  onNodesChange: OnNodesChange
  onEdgesChange: OnEdgesChange
  onConnect: OnConnect
  addNode: (node: Node) => void
  selectNode: (id: string | null) => void
  setGraph: (nodes: Node[], edges: Edge[]) => void
  setNodeTypes: (types: NodeMetadata[]) => void
  updateNodeData: (id: string, newData: Record<string, unknown>) => void

  reset: () => void
}

export const useFlowBuilderStore = create<FlowBuilderState>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  nodeTypes: [],

  onNodesChange: (changes: NodeChange[]) => {
    set({
      nodes: applyNodeChanges(changes, get().nodes),
    })
  },

  onEdgesChange: (changes: EdgeChange[]) => {
    set({
      edges: applyEdgeChanges(changes, get().edges),
    })
  },

  onConnect: (connection: Connection) => {
    set({
      edges: addEdge(connection, get().edges),
    })
  },

  addNode: (node: Node) => {
    set({ nodes: [...get().nodes, node] })
  },

  selectNode: (id: string | null) => {
    set({ selectedNodeId: id })
  },

  setGraph: (nodes: Node[], edges: Edge[]) => {
    set({ nodes, edges })
  },

  setNodeTypes: (types: NodeMetadata[]) => {
    set({ nodeTypes: types })
  },

  updateNodeData: (id: string, newData: Record<string, unknown>) => {
    set({
      nodes: get().nodes.map((node) => {
        if (node.id === id) {
          // Merge existing data with new data (important for preserving labels + config)
          return {
            ...node,
            data: { ...node.data, ...newData },
          }
        }
        return node
      }),
    })
  },

  reset: () => {
    set({ nodes: [], edges: [], selectedNodeId: null })
  },
}))
