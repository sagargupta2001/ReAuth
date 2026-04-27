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

export interface NodeCapabilities {
  supports_ui: boolean
  ui_surface?: 'form' | 'awaiting_action' | null
  allowed_page_categories?: Array<
    'auth' | 'consent' | 'awaiting_action' | 'verification' | 'mfa' | 'notification' | 'error' | 'custom'
  >
  async_pause?: boolean
  side_effects?: boolean
  requires_secrets?: boolean
}

export interface NodeContract {
  id: string
  category: string
  display_name: string
  description: string
  icon: string
  inputs: string[]
  outputs: string[]
  config_schema: Record<string, unknown>
  default_template_key?: string | null
  contract_version?: string
  capabilities: NodeCapabilities
}

export interface PublishIssue {
  message: string
  node_ids: string[]
}

export interface PublishError {
  message: string
  issues?: PublishIssue[]
}

interface FlowBuilderState {
  nodes: Node[]
  edges: Edge[]
  selectedNodeId: string | null
  nodeTypes: NodeContract[]
  publishError: PublishError | null
  onNodesChange: OnNodesChange
  onEdgesChange: OnEdgesChange
  onConnect: OnConnect
  addNode: (node: Node) => void
  selectNode: (id: string | null) => void
  setGraph: (nodes: Node[], edges: Edge[]) => void
  setNodeTypes: (types: NodeContract[]) => void
  updateNodeData: (id: string, newData: Record<string, unknown>) => void
  setPublishError: (error: PublishError | null) => void

  reset: () => void
}

export const useFlowBuilderStore = create<FlowBuilderState>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  nodeTypes: [],
  publishError: null,

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

  setNodeTypes: (types: NodeContract[]) => {
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

  setPublishError: (error: PublishError | null) => {
    set({ publishError: error })
  },

  reset: () => {
    set({ nodes: [], edges: [], selectedNodeId: null, publishError: null })
  },
}))
