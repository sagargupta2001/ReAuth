import { createEvent, fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { FluidBlocksPanel } from './FluidBlocksPanel'
import type { ThemeNode } from '@/entities/theme/model/types'
import { MAX_NESTING_DEPTH, SECTION_DRAG_MIME_TYPE } from '@/features/fluid/model/sectionTree'

const toastError = vi.hoisted(() => vi.fn())
vi.mock('sonner', () => ({ toast: { error: toastError, success: vi.fn() } }))

const textNode: ThemeNode = {
  id: 'node-text',
  type: 'Text',
  props: { text: 'Welcome back' },
}

const inputNode: ThemeNode = {
  id: 'node-input',
  type: 'Component',
  component: 'Input',
  props: { label: 'Email' },
  slots: {
    prefix: { id: 'node-input-prefix', type: 'Icon', props: { name: 'mail' } },
  },
}

function renderPanel(overrides: Partial<Parameters<typeof FluidBlocksPanel>[0]> = {}) {
  const props = {
    nodes: [textNode, inputNode],
    selectedNodeId: null,
    onSelectNode: vi.fn(),
    onInsertNode: vi.fn(),
    onRemoveNode: vi.fn(),
    onMoveNode: vi.fn(),
    ...overrides,
  }
  const view = render(<FluidBlocksPanel {...props} />)
  return { ...props, view }
}

/** The draggable row that owns a label. */
function rowFor(label: string): HTMLElement {
  const row = screen.getByText(label).closest('[draggable]')
  expect(row).not.toBeNull()
  return row as HTMLElement
}

const ROW_HEIGHT = 40

/**
 * jsdom gives every element a zero-sized rect, so a test that cares which third
 * of a row the pointer is in has to supply the geometry itself.
 */
function stubRect(element: HTMLElement, top = 0, height = ROW_HEIGHT) {
  element.getBoundingClientRect = () =>
    ({
      top,
      bottom: top + height,
      height,
      left: 0,
      right: 200,
      width: 200,
      x: 0,
      y: top,
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
 * jsdom has no `DragEvent`, so `fireEvent.dragOver(el, { clientY })` drops the
 * pointer position on the floor. Defining it on the native event is what lets a
 * test say which third of a row it is aiming at.
 */
function fireDragAt(
  type: 'dragOver' | 'drop',
  element: HTMLElement,
  dataTransfer: unknown,
  clientY: number,
) {
  const event = createEvent[type](element, { dataTransfer })
  Object.defineProperty(event, 'clientY', { value: clientY })
  return fireEvent(element, event)
}

type Zone = 'top' | 'middle' | 'bottom'

const OFFSETS: Record<Zone, number> = {
  top: 2,
  middle: ROW_HEIGHT / 2,
  bottom: ROW_HEIGHT - 2,
}

/** Drags `source` onto a zone of `target` and drops it there. */
function dragOnto(source: HTMLElement, target: HTMLElement, zone: Zone = 'middle') {
  const dataTransfer = makeDataTransfer()
  stubRect(target)
  fireEvent.dragStart(source, { dataTransfer })
  fireDragAt('dragOver', target, dataTransfer, OFFSETS[zone])
  fireDragAt('drop', target, dataTransfer, OFFSETS[zone])
  return dataTransfer
}

beforeEach(() => {
  toastError.mockClear()
})

describe('FluidBlocksPanel', () => {
  it('renders the page scaffold and each root node', () => {
    renderPanel()
    expect(screen.getByText('Page')).toBeInTheDocument()
    expect(screen.getByText('Layout Container')).toBeInTheDocument()
    expect(screen.getByText('Text')).toBeInTheDocument()
    expect(screen.getByText('Input Field')).toBeInTheDocument()
  })

  it('renders slot nodes under their owner', () => {
    renderPanel()
    expect(screen.getByText('Slot: prefix')).toBeInTheDocument()
    expect(screen.getByText('Icon')).toBeInTheDocument()
  })

  it('indents a child deeper than its parent', () => {
    renderPanel({
      nodes: [
        {
          id: 'node-box',
          type: 'Box',
          children: [{ id: 'node-child', type: 'Text', props: { text: 'Inner' } }],
        },
      ],
    })

    const indentOf = (label: string) => {
      const row = screen.getByText(label).closest('[style*="padding-left"]') as HTMLElement
      return Number.parseFloat(row.style.paddingLeft)
    }

    // The grip used to render only on root rows, stripping ~22px from child
    // rows, so children appeared further left than their parent.
    expect(indentOf('Text')).toBeGreaterThan(indentOf('Box'))
  })

  it('reserves the grip column on child rows so indentation reads correctly', () => {
    renderPanel({
      nodes: [
        {
          id: 'node-box',
          type: 'Box',
          children: [{ id: 'node-child', type: 'Text', props: { text: 'Inner' } }],
        },
      ],
    })

    const labelOffset = (label: string) =>
      screen.getByText(label).closest('button')?.previousElementSibling
    // Every row has the reserved slot, draggable or not.
    expect(labelOffset('Box')).not.toBeNull()
    expect(labelOffset('Text')).not.toBeNull()
  })

  it('shows an empty state when the page has no nodes', () => {
    renderPanel({ nodes: [] })
    expect(screen.getByText('Add blocks to build this page.')).toBeInTheDocument()
  })

  it('selects a node when its row is clicked', () => {
    const { onSelectNode } = renderPanel()
    fireEvent.click(screen.getByText('Text'))
    expect(onSelectNode).toHaveBeenCalledWith('node-text')
  })

  it('removes a node from its row action', () => {
    const { onRemoveNode } = renderPanel()
    fireEvent.click(screen.getByLabelText('Remove Input Field'))
    expect(onRemoveNode).toHaveBeenCalledWith('node-input')
  })

  it('inserts a picked block at the end when opened from the header', () => {
    const onInsertNode = vi.fn()
    renderPanel({ onInsertNode })

    fireEvent.click(screen.getByLabelText('Add block'))
    fireEvent.click(screen.getByText('Divider'))

    expect(onInsertNode).toHaveBeenCalledTimes(1)
    const [node, location] = onInsertNode.mock.calls[0]
    expect(node).toMatchObject({ type: 'Component', component: 'Divider' })
    expect(location).toEqual({ parentId: null, index: 2 })
  })

  it('inserts a picked block after the row it was opened from', () => {
    const onInsertNode = vi.fn()
    renderPanel({ onInsertNode })

    fireEvent.click(screen.getByLabelText('Add block after Text'))
    fireEvent.click(screen.getByText('Button'))

    expect(onInsertNode.mock.calls[0][1]).toEqual({ parentId: null, index: 1 })
  })

  it('filters the picker catalog by search query', () => {
    renderPanel()
    fireEvent.click(screen.getByLabelText('Add block'))

    fireEvent.change(screen.getByLabelText('Search blocks'), { target: { value: 'divid' } })

    expect(screen.getByText('Divider')).toBeInTheDocument()
    expect(screen.queryByText('Brand or hero image')).not.toBeInTheDocument()
  })

  it('closes the picker after a block is inserted', () => {
    renderPanel()
    fireEvent.click(screen.getByLabelText('Add block'))
    expect(screen.getByLabelText('Search blocks')).toBeInTheDocument()

    fireEvent.click(screen.getByText('Divider'))

    expect(screen.queryByLabelText('Search blocks')).not.toBeInTheDocument()
  })

  it('surfaces page-level validation errors', () => {
    renderPanel({
      validationErrors: [{ path: 'blueprint', message: 'Blueprint must define a nodes array.' }],
    })
    expect(
      screen.getByText('1 page-level validation issue(s). Check the inspector.'),
    ).toBeInTheDocument()
  })

  it('counts node-level validation errors on the node row', () => {
    renderPanel({
      validationErrors: [
        { path: 'nodes[0]', message: 'Node type is required.', nodeId: 'node-text' },
        { path: 'nodes[0]', message: 'Unsupported node type', nodeId: 'node-text' },
      ],
    })
    expect(screen.getByTitle('2 validation issue(s)')).toBeInTheDocument()
  })
})

describe('FluidBlocksPanel nesting', () => {
  const emptyBox: ThemeNode = { id: 'node-box', type: 'Box', children: [] }

  it('nests a node dropped on the middle of a container row', () => {
    const { onMoveNode } = renderPanel({ nodes: [textNode, emptyBox] })

    dragOnto(rowFor('Text'), rowFor('Box'), 'middle')

    expect(onMoveNode).toHaveBeenCalledWith('node-text', { parentId: 'node-box', index: 0 })
  })

  it('carries the node id, not its index, in the drag payload', () => {
    renderPanel({ nodes: [textNode, emptyBox] })
    const dataTransfer = dragOnto(rowFor('Text'), rowFor('Box'))
    expect(dataTransfer.getData(SECTION_DRAG_MIME_TYPE)).toBe('node-text')
  })

  it('reorders within a container when dropped on a sibling top edge', () => {
    const { onMoveNode } = renderPanel({
      nodes: [
        {
          id: 'node-box',
          type: 'Box',
          children: [
            { id: 'child-text', type: 'Text', props: { text: 'Inner' } },
            { id: 'child-link', type: 'Component', component: 'Link', props: { label: 'Go' } },
          ],
        },
      ],
    })

    dragOnto(rowFor('Link'), rowFor('Text'), 'top')

    expect(onMoveNode).toHaveBeenCalledWith('child-link', { parentId: 'node-box', index: 0 })
  })

  it('moves a child out to the page root when dropped on a root sibling', () => {
    const { onMoveNode } = renderPanel({
      nodes: [
        { id: 'node-box', type: 'Box', children: [{ id: 'child-text', type: 'Text' }] },
        { id: 'node-tail', type: 'Component', component: 'Divider' },
      ],
    })

    dragOnto(rowFor('Text'), rowFor('Divider'), 'bottom')

    expect(onMoveNode).toHaveBeenCalledWith('child-text', { parentId: null, index: 2 })
  })

  it('renders a drop target inside an empty container', () => {
    const { onMoveNode } = renderPanel({ nodes: [textNode, emptyBox] })
    const zone = screen.getByText('Drop a block here')

    const dataTransfer = makeDataTransfer()
    fireEvent.dragStart(rowFor('Text'), { dataTransfer })
    fireEvent.dragOver(zone, { dataTransfer })
    fireEvent.drop(zone, { dataTransfer })

    expect(onMoveNode).toHaveBeenCalledWith('node-text', { parentId: 'node-box', index: 0 })
  })

  it('keeps the empty-container target on a container whose last child left', () => {
    const { view } = renderPanel({ nodes: [textNode, emptyBox] })
    expect(screen.getByText('Drop a block here')).toBeInTheDocument()

    view.rerender(
      <FluidBlocksPanel
        nodes={[{ ...emptyBox, children: [textNode] }]}
        selectedNodeId={null}
        onSelectNode={vi.fn()}
        onInsertNode={vi.fn()}
        onRemoveNode={vi.fn()}
        onMoveNode={vi.fn()}
      />,
    )
    expect(screen.queryByText('Drop a block here')).not.toBeInTheDocument()
  })

  it('marks the active drop zone while dragging', () => {
    renderPanel({ nodes: [textNode, emptyBox] })
    const dataTransfer = makeDataTransfer()
    const target = rowFor('Box')
    stubRect(target)

    fireEvent.dragStart(rowFor('Text'), { dataTransfer })
    fireDragAt('dragOver', target, dataTransfer, OFFSETS.middle)

    expect(target).toHaveAttribute('data-drop-intent', 'inside')
    expect(target).toHaveAttribute('data-drop-allowed', 'true')
  })

  it('inserts a picked block inside the container it was opened from', () => {
    const onInsertNode = vi.fn()
    renderPanel({ nodes: [textNode, emptyBox], onInsertNode })

    fireEvent.click(screen.getByLabelText('Add block inside Box'))
    fireEvent.click(screen.getByText('Divider'))

    expect(onInsertNode.mock.calls[0][1]).toEqual({ parentId: 'node-box', index: 0 })
  })

  it('offers no inside-container affordance on a block that cannot hold children', () => {
    renderPanel()
    expect(screen.queryByLabelText('Add block inside Text')).not.toBeInTheDocument()
  })

  it('collapses a container so a deep tree stays navigable', () => {
    renderPanel({
      nodes: [{ id: 'node-box', type: 'Box', children: [{ id: 'child-text', type: 'Text' }] }],
    })

    fireEvent.click(screen.getByLabelText('Collapse Box'))
    expect(screen.queryByText('Text')).not.toBeInTheDocument()

    fireEvent.click(screen.getByLabelText('Expand Box'))
    expect(screen.getByText('Text')).toBeInTheDocument()
  })
})

describe('FluidBlocksPanel rejected drops', () => {
  it('refuses to move a container into its own descendant', () => {
    const { onMoveNode } = renderPanel({
      nodes: [
        {
          id: 'box-a',
          type: 'Box',
          children: [{ id: 'box-b', type: 'Box', children: [] }],
        },
      ],
    })
    const rows = screen.getAllByText('Box')
    const outer = rows[0].closest('[draggable="true"]') as HTMLElement
    const inner = rows[1].closest('[draggable="true"]') as HTMLElement

    stubRect(inner)
    const dataTransfer = makeDataTransfer()
    fireEvent.dragStart(outer, { dataTransfer })
    fireDragAt('dragOver', inner, dataTransfer, OFFSETS.middle)

    expect(inner).toHaveAttribute('data-drop-allowed', 'false')

    fireDragAt('drop', inner, dataTransfer, OFFSETS.middle)
    expect(onMoveNode).not.toHaveBeenCalled()
    expect(toastError).toHaveBeenCalledWith('A block cannot be moved inside itself.')
  })

  it('treats a drop on a non-container as a sibling drop, never as nesting', () => {
    const { onMoveNode } = renderPanel({
      nodes: [{ id: 'node-link', type: 'Component', component: 'Link' }, textNode],
    })

    dragOnto(rowFor('Link'), rowFor('Text'), 'middle')

    // A non-container row has no middle band: the lower half means "after", and
    // the node stays a sibling instead of becoming a child of the Text.
    expect(onMoveNode).toHaveBeenCalledWith('node-link', { parentId: null, index: 2 })
  })

  it('rejects a drop past the nesting limit and says why', () => {
    let chain: ThemeNode = { id: `box-${MAX_NESTING_DEPTH - 1}`, type: 'Box', children: [] }
    for (let level = MAX_NESTING_DEPTH - 2; level >= 0; level -= 1) {
      chain = { id: `box-${level}`, type: 'Box', children: [chain] }
    }
    const { onMoveNode } = renderPanel({ nodes: [chain, textNode] })

    const deepest = screen.getAllByText('Box').at(-1)!.closest('[draggable="true"]') as HTMLElement
    dragOnto(rowFor('Text'), deepest, 'middle')

    expect(onMoveNode).not.toHaveBeenCalled()
    expect(toastError).toHaveBeenCalledWith(
      `Blocks can be nested up to ${MAX_NESTING_DEPTH} levels deep.`,
    )
  })

  it('records nothing when a node is dropped back where it already was', () => {
    const { onMoveNode } = renderPanel()

    dragOnto(rowFor('Text'), rowFor('Input Field'), 'top')

    expect(onMoveNode).not.toHaveBeenCalled()
    expect(toastError).not.toHaveBeenCalled()
  })

  it('discards a drag whose node was removed before the drop', () => {
    const { onMoveNode, view } = renderPanel()
    const dataTransfer = makeDataTransfer()
    fireEvent.dragStart(rowFor('Text'), { dataTransfer })

    view.rerender(
      <FluidBlocksPanel
        nodes={[inputNode]}
        selectedNodeId={null}
        onSelectNode={vi.fn()}
        onInsertNode={vi.fn()}
        onRemoveNode={vi.fn()}
        onMoveNode={onMoveNode}
      />,
    )

    const target = rowFor('Input Field')
    stubRect(target)
    expect(() => fireDragAt('drop', target, dataTransfer, OFFSETS.bottom)).not.toThrow()
    expect(onMoveNode).not.toHaveBeenCalled()
  })
})

describe('FluidBlocksPanel slot rows', () => {
  it('does not let a slot row be dragged', () => {
    renderPanel()
    expect(rowFor('Icon')).toHaveAttribute('draggable', 'false')
  })

  it('does not let a node be dropped on a slot row', () => {
    const { onMoveNode } = renderPanel()

    dragOnto(rowFor('Text'), rowFor('Icon'), 'middle')

    expect(onMoveNode).not.toHaveBeenCalled()
  })

  it('gives a slot row no insertion affordance', () => {
    renderPanel()
    expect(screen.queryByLabelText('Add block after Icon')).not.toBeInTheDocument()
  })
})

describe('FluidBlocksPanel keyboard', () => {
  const boxThenText: ThemeNode[] = [
    { id: 'node-box', type: 'Box', children: [{ id: 'child-text', type: 'Text' }] },
    { id: 'node-divider', type: 'Component', component: 'Divider' },
  ]

  it('indents a node into the container above it on Tab', () => {
    const { onMoveNode } = renderPanel({ nodes: boxThenText })

    fireEvent.keyDown(screen.getByText('Divider'), { key: 'Tab' })

    // Matches the drag result: last child of the preceding container.
    expect(onMoveNode).toHaveBeenCalledWith('node-divider', { parentId: 'node-box', index: 1 })
  })

  it('outdents a child to its parent level on Shift+Tab', () => {
    const { onMoveNode } = renderPanel({ nodes: boxThenText })

    fireEvent.keyDown(screen.getByText('Text'), { key: 'Tab', shiftKey: true })

    expect(onMoveNode).toHaveBeenCalledWith('child-text', { parentId: null, index: 1 })
  })

  it('reorders within the current parent on Alt+Arrow', () => {
    const onMoveNode = vi.fn()
    renderPanel({ onMoveNode })

    fireEvent.keyDown(screen.getByText('Input Field'), { key: 'ArrowUp', altKey: true })
    expect(onMoveNode).toHaveBeenCalledWith('node-input', { parentId: null, index: 0 })

    onMoveNode.mockClear()
    fireEvent.keyDown(screen.getByText('Text'), { key: 'ArrowDown', altKey: true })
    expect(onMoveNode).toHaveBeenCalledWith('node-text', { parentId: null, index: 2 })
  })

  it('leaves Tab alone when there is nothing to indent into', () => {
    // Otherwise the tree would be a keyboard trap: every Tab consumed, with no
    // way out of the panel.
    const { onMoveNode } = renderPanel()
    const event = fireEvent.keyDown(screen.getByText('Text'), { key: 'Tab' })

    expect(event).toBe(true)
    expect(onMoveNode).not.toHaveBeenCalled()
  })

  it('leaves Alt+Arrow alone at the ends of a sibling list', () => {
    const { onMoveNode } = renderPanel()
    fireEvent.keyDown(screen.getByText('Text'), { key: 'ArrowUp', altKey: true })
    fireEvent.keyDown(screen.getByText('Input Field'), { key: 'ArrowDown', altKey: true })
    expect(onMoveNode).not.toHaveBeenCalled()
  })
})
