import { createEvent, fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidCanvas } from './FluidCanvas'
import type { ThemeNode } from '@/entities/theme/model/types'
import { useFluidDrag } from '@/features/fluid/hooks/useFluidDrag'
import type { NodeLocation } from '@/features/fluid/lib/nodeUtils'

vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }))

/**
 * Canvas drag reuses `resolveDrop` / `moveNode` through the builder's shared
 * drag session, so what needs asserting here is the canvas-side half: which
 * nodes participate, and which axis its drop edges follow.
 */
function Harness({
  nodes,
  onMoveNode,
  onInsertNode,
  isInspecting = true,
  onReady,
}: {
  nodes: ThemeNode[]
  onMoveNode: (nodeId: string, location: NodeLocation) => void
  onInsertNode?: (node: ThemeNode, location: NodeLocation) => void
  isInspecting?: boolean
  onReady?: (drag: ReturnType<typeof useFluidDrag>) => void
}) {
  const drag = useFluidDrag(nodes, onMoveNode, onInsertNode)
  onReady?.(drag)
  return (
    <FluidCanvas
      tokens={{ colors: { primary: '#111827' } }}
      layout={{ shell: 'CenteredCard' }}
      blocks={nodes}
      assets={[]}
      selectedNodeId={null}
      isInspecting={isInspecting}
      drag={drag}
      onSelectNode={vi.fn()}
    />
  )
}

function renderCanvas(nodes: ThemeNode[], isInspecting = true) {
  const onMoveNode = vi.fn()
  const onInsertNode = vi.fn()
  let controller: ReturnType<typeof useFluidDrag> | undefined
  render(
    <Harness
      nodes={nodes}
      onMoveNode={onMoveNode}
      onInsertNode={onInsertNode}
      isInspecting={isInspecting}
      onReady={(drag) => {
        controller = drag
      }}
    />,
  )
  return { onMoveNode, onInsertNode, controller: () => controller! }
}

/** The draggable wrapper the walker put around a node. */
function nodeEl(text: string): HTMLElement {
  const el = screen.getByText(text).closest('[draggable]')
  expect(el).not.toBeNull()
  return el as HTMLElement
}

function stubRect(element: HTMLElement, rect: Partial<DOMRect>) {
  const full = { left: 0, top: 0, width: 200, height: 40, ...rect }
  element.getBoundingClientRect = () =>
    ({
      ...full,
      right: full.left + full.width,
      bottom: full.top + full.height,
      x: full.left,
      y: full.top,
      toJSON: () => ({}),
    }) as DOMRect
}

function makeDataTransfer() {
  const store = new Map<string, string>()
  return {
    setData: (type: string, value: string) => store.set(type, value),
    getData: (type: string) => store.get(type) ?? '',
    effectAllowed: '',
    dropEffect: '',
  }
}

/**
 * jsdom has no `DragEvent`, so `fireEvent` drops the pointer coordinates. They
 * have to be defined on the native event for the geometry to be exercised.
 */
function fireDragAt(
  type: 'dragOver' | 'drop',
  element: HTMLElement,
  dataTransfer: unknown,
  point: { clientX: number; clientY: number },
) {
  const event = createEvent[type](element, { dataTransfer })
  Object.defineProperty(event, 'clientX', { value: point.clientX })
  Object.defineProperty(event, 'clientY', { value: point.clientY })
  return fireEvent(element, event)
}

function dragOnto(
  source: HTMLElement,
  target: HTMLElement,
  point: { clientX: number; clientY: number },
) {
  const dataTransfer = makeDataTransfer()
  fireEvent.dragStart(source, { dataTransfer })
  fireDragAt('dragOver', target, dataTransfer, point)
  fireDragAt('drop', target, dataTransfer, point)
}

const COLUMN_NODES: ThemeNode[] = [
  { id: 'a', type: 'Text', props: { text: 'First' } },
  { id: 'b', type: 'Text', props: { text: 'Second' } },
]

describe('canvas drag', () => {
  it('reorders siblings when dropped on an edge', () => {
    const { onMoveNode } = renderCanvas(COLUMN_NODES)
    const target = nodeEl('First')
    stubRect(target, { top: 0, height: 40 })

    dragOnto(nodeEl('Second'), target, { clientX: 100, clientY: 2 })

    expect(onMoveNode).toHaveBeenCalledWith('b', { parentId: null, index: 0 })
  })

  it('nests when dropped in the middle of a container', () => {
    const { onMoveNode } = renderCanvas([
      { id: 'txt', type: 'Text', props: { text: 'Loose' } },
      {
        id: 'box',
        type: 'Box',
        children: [{ id: 'inner', type: 'Text', props: { text: 'Inside' } }],
      },
    ])
    const target = screen.getByText('Inside').closest('[draggable]')!
      .parentElement!.closest('[draggable]') as HTMLElement
    stubRect(target, { top: 0, height: 100 })

    dragOnto(nodeEl('Loose'), target, { clientX: 100, clientY: 50 })

    expect(onMoveNode).toHaveBeenCalledWith('txt', { parentId: 'box', index: 1 })
  })

  it('uses the horizontal edges for a block inside a row', () => {
    const { onMoveNode } = renderCanvas([
      {
        id: 'row',
        type: 'Box',
        layout: { direction: 'row', gap: 8 },
        children: [
          { id: 'a', type: 'Text', props: { text: 'Alpha' } },
          { id: 'b', type: 'Text', props: { text: 'Beta' } },
          { id: 'c', type: 'Text', props: { text: 'Gamma' } },
        ],
      },
    ])
    const target = nodeEl('Alpha')
    stubRect(target, { left: 0, width: 100, top: 0, height: 40 })

    // A point the two axes disagree about: near Alpha's left edge (horizontally
    // "before") but near its bottom (vertically "after"). Reading the parent's
    // row axis lands Gamma at index 0; reading the tree's fixed vertical axis
    // would land it at index 1.
    dragOnto(nodeEl('Gamma'), target, { clientX: 10, clientY: 38 })

    expect(onMoveNode).toHaveBeenCalledWith('c', { parentId: 'row', index: 0 })
  })

  it('marks the active drop zone while dragging', () => {
    renderCanvas(COLUMN_NODES)
    const target = nodeEl('First')
    stubRect(target, { top: 0, height: 40 })
    const dataTransfer = makeDataTransfer()

    fireEvent.dragStart(nodeEl('Second'), { dataTransfer })
    fireDragAt('dragOver', target, dataTransfer, { clientX: 100, clientY: 2 })

    expect(target).toHaveAttribute('data-drop-intent', 'before')
    expect(target).toHaveAttribute('data-drop-allowed', 'true')
  })

  it('refuses to drop a container into its own descendant', () => {
    const { onMoveNode } = renderCanvas([
      {
        id: 'outer',
        type: 'Box',
        children: [{ id: 'inner', type: 'Box', children: [] }],
      },
      { id: 'tail', type: 'Text', props: { text: 'Tail' } },
    ])
    const boxes = document.querySelectorAll<HTMLElement>('[draggable="true"]')
    const outer = boxes[0]
    const inner = boxes[1]
    stubRect(inner, { top: 0, height: 60 })

    const dataTransfer = makeDataTransfer()
    fireEvent.dragStart(outer, { dataTransfer })
    fireDragAt('dragOver', inner, dataTransfer, { clientX: 100, clientY: 30 })

    expect(inner).toHaveAttribute('data-drop-allowed', 'false')

    fireDragAt('drop', inner, dataTransfer, { clientX: 100, clientY: 30 })
    expect(onMoveNode).not.toHaveBeenCalled()
  })

  it('leaves a Component expansion out of the drag entirely', () => {
    renderCanvas([
      {
        id: 'field',
        type: 'Component',
        component: 'Input',
        props: { label: 'Email', name: 'email', placeholder: 'you@example.com' },
      },
    ])

    // The authored Input is the only drag source. Its generated label, field
    // container and inner input render inside that one wrapper rather than
    // becoming drag sources of their own.
    const draggables = document.querySelectorAll('[draggable="true"]')
    expect(draggables).toHaveLength(1)
    expect(draggables[0].contains(screen.getByText('Email'))).toBe(true)
    // In inspect mode the canvas renders the field as inert text, so the
    // placeholder is content rather than an attribute.
    expect(draggables[0].contains(screen.getByText('you@example.com'))).toBe(true)
  })

  it('makes nothing draggable outside inspect mode', () => {
    renderCanvas(COLUMN_NODES, false)
    expect(document.querySelectorAll('[draggable="true"]')).toHaveLength(0)
  })

  it('makes nothing draggable without a drag session', () => {
    // The preview surfaces render the same canvas with no controller.
    render(
      <FluidCanvas
        tokens={{}}
        layout={{ shell: 'CenteredCard' }}
        blocks={COLUMN_NODES}
        assets={[]}
        selectedNodeId={null}
        isInspecting
        onSelectNode={vi.fn()}
      />,
    )
    expect(document.querySelectorAll('[draggable="true"]')).toHaveLength(0)
  })
})

describe('canvas drag from the picker', () => {
  it('places a new block where it is dropped on the canvas', () => {
    const { onInsertNode, onMoveNode, controller } = renderCanvas(COLUMN_NODES)
    const target = nodeEl('First')
    stubRect(target, { top: 0, height: 40 })

    // The picker lives in the other panel; what reaches the canvas is a drag
    // the shared session already knows about.
    const dataTransfer = makeDataTransfer()
    controller().onBlockDragStart(
      { dataTransfer, currentTarget: target } as never,
      'divider',
    )
    fireDragAt('dragOver', target, dataTransfer, { clientX: 100, clientY: 2 })
    fireDragAt('drop', target, dataTransfer, { clientX: 100, clientY: 2 })

    expect(onMoveNode).not.toHaveBeenCalled()
    expect(onInsertNode).toHaveBeenCalledTimes(1)
    const [node, location] = onInsertNode.mock.calls[0]
    expect(node).toMatchObject({ type: 'Component', component: 'Divider' })
    expect(location).toEqual({ parentId: null, index: 0 })
  })
})
