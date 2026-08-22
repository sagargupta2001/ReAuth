import type { Node } from '@xyflow/react'
import { beforeEach, describe, expect, it } from 'vitest'

import { useFlowBuilderStore } from './flowBuilderStore'

const makeNode = (id: string): Node => ({
  id,
  position: { x: 0, y: 0 },
  data: {},
  type: 'logic',
})

describe('flowBuilderStore history', () => {
  beforeEach(() => {
    useFlowBuilderStore.getState().reset()
  })

  it('records discrete actions and undoes/redoes them', () => {
    const { addNode, undo, redo } = useFlowBuilderStore.getState()

    addNode(makeNode('a'))
    addNode(makeNode('b'))
    expect(useFlowBuilderStore.getState().nodes).toHaveLength(2)
    expect(useFlowBuilderStore.getState().past).toHaveLength(2)

    undo()
    expect(useFlowBuilderStore.getState().nodes.map((n) => n.id)).toEqual(['a'])
    expect(useFlowBuilderStore.getState().future).toHaveLength(1)

    redo()
    expect(useFlowBuilderStore.getState().nodes.map((n) => n.id)).toEqual(['a', 'b'])
    expect(useFlowBuilderStore.getState().future).toHaveLength(0)
  })

  it('clears history when a draft graph is loaded', () => {
    const { addNode, setGraph } = useFlowBuilderStore.getState()

    addNode(makeNode('a'))
    expect(useFlowBuilderStore.getState().past).toHaveLength(1)

    setGraph([makeNode('loaded')], [])
    expect(useFlowBuilderStore.getState().past).toHaveLength(0)
    expect(useFlowBuilderStore.getState().future).toHaveLength(0)
  })

  it('does not record selection-only node changes', () => {
    const { setGraph, onNodesChange } = useFlowBuilderStore.getState()

    setGraph([makeNode('a')], [])
    onNodesChange([{ type: 'select', id: 'a', selected: true }])
    expect(useFlowBuilderStore.getState().past).toHaveLength(0)
  })

  it('records a single history entry for a drag interaction', () => {
    const { setGraph, onNodesChange } = useFlowBuilderStore.getState()

    setGraph([makeNode('a')], [])
    onNodesChange([{ type: 'position', id: 'a', position: { x: 5, y: 5 }, dragging: true }])
    onNodesChange([{ type: 'position', id: 'a', position: { x: 9, y: 9 }, dragging: true }])
    onNodesChange([{ type: 'position', id: 'a', position: { x: 9, y: 9 }, dragging: false }])

    expect(useFlowBuilderStore.getState().past).toHaveLength(1)

    useFlowBuilderStore.getState().undo()
    expect(useFlowBuilderStore.getState().nodes[0].position).toEqual({ x: 0, y: 0 })
  })
})
