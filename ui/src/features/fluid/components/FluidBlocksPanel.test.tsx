import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidBlocksPanel } from './FluidBlocksPanel'
import type { ThemeNode } from '@/entities/theme/model/types'

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
    onReorderNodes: vi.fn(),
    ...overrides,
  }
  render(<FluidBlocksPanel {...props} />)
  return props
}

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
    const [node, index] = onInsertNode.mock.calls[0]
    expect(node).toMatchObject({ type: 'Component', component: 'Divider' })
    expect(index).toBe(2)
  })

  it('inserts a picked block after the row it was opened from', () => {
    const onInsertNode = vi.fn()
    renderPanel({ onInsertNode })

    fireEvent.click(screen.getByLabelText('Add block after Text'))
    fireEvent.click(screen.getByText('Button'))

    expect(onInsertNode.mock.calls[0][1]).toBe(1)
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

  it('reorders nodes on drop', () => {
    const { onReorderNodes } = renderPanel()
    const transfer = new Map<string, string>()
    const dataTransfer = {
      setData: (type: string, value: string) => transfer.set(type, value),
      getData: (type: string) => transfer.get(type) ?? '',
      effectAllowed: '',
      dropEffect: '',
    }

    const rows = screen.getByText('Text').closest('[draggable="true"]')
    const target = screen.getByText('Input Field').closest('[draggable="true"]')
    expect(rows).not.toBeNull()
    expect(target).not.toBeNull()

    fireEvent.dragStart(rows!, { dataTransfer })
    fireEvent.drop(target!, { dataTransfer })

    expect(onReorderNodes).toHaveBeenCalledWith(0, 1)
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
