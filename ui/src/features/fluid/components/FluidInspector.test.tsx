import { fireEvent, render, screen, within } from '@testing-library/react'
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

    fireEvent.change(screen.getByLabelText('Background'), { target: { value: '#101828' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({
      style: { group: 'fill', key: 'color', part: undefined, value: '#101828' },
    })

    fireEvent.change(screen.getByLabelText('Corner Radius'), { target: { value: '16' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({
      style: { group: 'corners', key: 'radius', part: undefined, value: 16 },
    })
  })

  it('writes an Input part style to the part, not to a prefixed prop', () => {
    // `label_color`, `field_radius` and the seven others like them are gone.
    const { onUpdateSelectedBlock } = renderInspector({
      id: 'i1',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email', name: 'email' },
    })

    fireEvent.change(screen.getByLabelText('Label Color'), { target: { value: '#8899aa' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({
      style: { group: 'typography', key: 'color', part: 'label', value: '#8899aa' },
    })

    fireEvent.change(screen.getByLabelText('Inner Padding'), { target: { value: '12' } })
    expect(onUpdateSelectedBlock).toHaveBeenCalledWith({
      style: { group: 'spacing', key: 'padding', part: 'field', value: 12 },
    })
  })

  it('reads a part style back from the legacy prop it replaced', () => {
    renderInspector({
      id: 'i2',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email', name: 'email', field_border_width: 3 },
    })
    expect(screen.getByLabelText('Border Width')).toHaveValue(3)
  })

  it('offers block colours as a design token or a literal, not a raw string', () => {
    // Every inspector colour used to be a plain text input, so a block colour
    // could only ever be pinned to a literal. Referencing a token is what lets
    // a palette change propagate.
    renderInspector({
      id: 'b2',
      type: 'Box',
      props: { background: 'var(--card)' },
      children: [],
    })

    const source = screen.getByRole('group', { name: 'Background source' })
    expect(source).toBeInTheDocument()

    // A token-valued colour resolves to the token picker rather than a swatch
    // showing an unrelated fallback.
    const tokenButton = within(source).getByRole('button', { name: 'Design token' })
    expect(tokenButton).toHaveAttribute('aria-pressed', 'true')
    expect(within(source).getByRole('button', { name: 'Custom' })).toHaveAttribute(
      'aria-pressed',
      'false',
    )
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
