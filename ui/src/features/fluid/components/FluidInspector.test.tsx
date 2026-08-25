import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidInspector } from './FluidInspector'
import type { ThemeNode } from '@/entities/theme/model/types'

function renderInspector(
  selectedBlock: ThemeNode | null,
  overrides: Partial<Parameters<typeof FluidInspector>[0]> = {},
) {
  const onUpdateSelectedBlock = vi.fn()
  render(
    <FluidInspector
      assets={[]}
      tokens={{ colors: { text: '#ededed', background: '#000000' } }}
      selectedBlock={selectedBlock}
      inputNames={['email']}
      onUpdateSelectedBlock={onUpdateSelectedBlock}
      {...overrides}
    />,
  )
  return { onUpdateSelectedBlock }
}

const box: ThemeNode = {
  id: 'b1',
  type: 'Box',
  layout: { direction: 'column', gap: 12, padding: [4, 4, 4, 4] },
  size: { width: 'fill', height: 'hug' },
  children: [],
}

describe('FluidInspector', () => {
  it('prompts when nothing is selected', () => {
    renderInspector(null)
    expect(screen.getByText(/Select a block from the canvas/i)).toBeInTheDocument()
  })

  it('shows the sections that apply to a Box', () => {
    renderInspector(box)

    expect(screen.getByText('Auto Layout')).toBeInTheDocument()
    expect(screen.getByText('Size')).toBeInTheDocument()
    expect(screen.getByText('Spacing')).toBeInTheDocument()
    // Typography made no sense on a Box and used to render regardless.
    expect(screen.queryByText('Typography')).not.toBeInTheDocument()
  })

  it('shows typography for a Text node', () => {
    renderInspector({ id: 't1', type: 'Text', props: { text: 'Hi' } })
    expect(screen.getByText('Typography')).toBeInTheDocument()
  })

  it('writes box direction to layout, not props', () => {
    const { onUpdateSelectedBlock } = renderInspector(box)

    fireEvent.change(screen.getByLabelText('Gap'), { target: { value: '24' } })

    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({ layout: { gap: 24 } })
  })

  it('writes text alignment to props', () => {
    const { onUpdateSelectedBlock } = renderInspector(box)

    // Distinct label from "Align children", which targets layout.align.
    expect(screen.getByText('Text alignment')).toBeInTheDocument()
    expect(screen.getByText('Align children')).toBeInTheDocument()
    expect(onUpdateSelectedBlock).not.toHaveBeenCalled()
  })

  it('writes the padding tuple as all four sides', () => {
    const { onUpdateSelectedBlock } = renderInspector(box)

    fireEvent.change(screen.getByLabelText('Inner padding top'), { target: { value: '10' } })

    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({
      layout: { padding: [10, 4, 4, 4] },
    })
  })

  it('writes size mode to node.size', () => {
    const { onUpdateSelectedBlock } = renderInspector({
      ...box,
      size: { width: 'fixed', width_value: '240px', height: 'hug' },
    })

    fireEvent.change(screen.getByLabelText('Custom Width'), { target: { value: '300' } })

    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({ size: { width_value: '300' } })
  })

  it('only offers the custom value when the mode is fixed', () => {
    renderInspector(box)
    expect(screen.queryByLabelText('Custom Width')).not.toBeInTheDocument()
  })

  it('exposes the box surface props both renderers already read', () => {
    // Background, border, and radius rendered from the blueprint long before
    // any control wrote them — the capability matrix is what surfaced that.
    const { onUpdateSelectedBlock } = renderInspector(box)

    expect(screen.getByText('Appearance')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('Background'), { target: { value: 'var(--card)' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({ props: { background: 'var(--card)' } })

    fireEvent.change(screen.getByLabelText('Corner Radius'), { target: { value: '16' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({ props: { radius: 16 } })
  })

  it('does not offer box surface props on a Text node', () => {
    renderInspector({ id: 't1', type: 'Text', props: { text: 'Hi' } })
    expect(screen.queryByText('Appearance')).not.toBeInTheDocument()
  })

  it('offers a placeholder control for an Input', () => {
    renderInspector({
      id: 'i1',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email', name: 'email' },
    })
    expect(screen.getByLabelText('Placeholder')).toBeInTheDocument()
  })

  it('offers the actions tab only for action-capable nodes', () => {
    renderInspector({ id: 'btn', type: 'Component', component: 'Button', props: {} })
    expect(screen.getByRole('button', { name: 'Actions' })).toBeInTheDocument()
  })

  it('hides the actions tab for a Box', () => {
    renderInspector(box)
    expect(screen.queryByRole('button', { name: 'Actions' })).not.toBeInTheDocument()
  })

  it('surfaces validation errors for the selected node', () => {
    renderInspector(box, {
      validationErrors: [{ path: 'nodes[0]', message: 'Node type is required.', nodeId: 'b1' }],
    })
    expect(screen.getByText('Node type is required.')).toBeInTheDocument()
  })
})
