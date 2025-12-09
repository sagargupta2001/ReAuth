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

interface FlowBuilderState {
  nodes: Node[]
  edges: Edge[]
  selectedNodeId: string | null

  // Actions
  onNodesChange: OnNodesChange
  onEdgesChange: OnEdgesChange
  onConnect: OnConnect
  addNode: (node: Node) => void
  selectNode: (id: string | null) => void
  setGraph: (nodes: Node[], edges: Edge[]) => void
}

export const useFlowBuilderStore = create<FlowBuilderState>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,

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
}))
