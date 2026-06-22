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

interface GraphSnapshot {
  nodes: Node[]
  edges: Edge[]
}

const MAX_HISTORY = 50

interface FlowBuilderState {
  nodes: Node[]
  edges: Edge[]
  selectedNodeId: string | null
  nodeTypes: NodeContract[]
  publishError: PublishError | null
  past: GraphSnapshot[]
  future: GraphSnapshot[]
  // Transient flag so a single drag interaction records only one history entry.
  isDragging: boolean
  onNodesChange: OnNodesChange
  onEdgesChange: OnEdgesChange
  onConnect: OnConnect
  addNode: (node: Node) => void
  selectNode: (id: string | null) => void
  setGraph: (nodes: Node[], edges: Edge[]) => void
  setNodeTypes: (types: NodeContract[]) => void
  updateNodeData: (id: string, newData: Record<string, unknown>) => void
  setPublishError: (error: PublishError | null) => void
  undo: () => void
  redo: () => void

  reset: () => void
}

export const useFlowBuilderStore = create<FlowBuilderState>((set, get) => {
  // Push the current graph onto the undo stack and clear the redo stack.
  const takeSnapshot = () => {
    const { nodes, edges, past } = get()
    set({
      past: [...past.slice(-(MAX_HISTORY - 1)), { nodes, edges }],
      future: [],
    })
  }

  return {
  nodes: [],
  edges: [],
  selectedNodeId: null,
  nodeTypes: [],
  publishError: null,
  past: [],
  future: [],
  isDragging: false,

  onNodesChange: (changes: NodeChange[]) => {
    const isRemoval = changes.some((change) => change.type === 'remove')
    const dragStart =
      !get().isDragging &&
      changes.some((change) => change.type === 'position' && change.dragging === true)
    const dragEnd = changes.some(
      (change) => change.type === 'position' && change.dragging === false,
    )

    // Snapshot the pre-change graph for removals and at the start of a drag.
    if (isRemoval || dragStart) {
      takeSnapshot()
    }

    set((state) => ({
      nodes: applyNodeChanges(changes, state.nodes),
      isDragging: dragStart ? true : dragEnd ? false : state.isDragging,
    }))
  },

  onEdgesChange: (changes: EdgeChange[]) => {
    if (changes.some((change) => change.type === 'remove')) {
      takeSnapshot()
    }
    set({
      edges: applyEdgeChanges(changes, get().edges),
    })
  },

  onConnect: (connection: Connection) => {
    takeSnapshot()
    set({
      edges: addEdge(connection, get().edges),
    })
  },

  addNode: (node: Node) => {
    takeSnapshot()
    set({ nodes: [...get().nodes, node] })
  },

  selectNode: (id: string | null) => {
    set({ selectedNodeId: id })
  },

  setGraph: (nodes: Node[], edges: Edge[]) => {
    // Loading a draft resets history so undo cannot reach an empty graph.
    set({ nodes, edges, past: [], future: [], isDragging: false })
  },

  setNodeTypes: (types: NodeContract[]) => {
    set({ nodeTypes: types })
  },

  updateNodeData: (id: string, newData: Record<string, unknown>) => {
    takeSnapshot()
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

  undo: () => {
    const { past, future, nodes, edges } = get()
    if (past.length === 0) return
    const previous = past[past.length - 1]
    set({
      nodes: previous.nodes,
      edges: previous.edges,
      past: past.slice(0, -1),
      future: [{ nodes, edges }, ...future],
    })
  },

  redo: () => {
    const { past, future, nodes, edges } = get()
    if (future.length === 0) return
    const next = future[0]
    set({
      nodes: next.nodes,
      edges: next.edges,
      past: [...past, { nodes, edges }],
      future: future.slice(1),
    })
  },

  reset: () => {
    set({
      nodes: [],
      edges: [],
      selectedNodeId: null,
      publishError: null,
      past: [],
      future: [],
      isDragging: false,
    })
  },
  }
})
