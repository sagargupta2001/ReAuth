import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidCanvas } from './FluidCanvas'
import type { ThemeNode } from '@/entities/theme/model/types'

/**
 * Regression tests for renderer bugs that made theme props silently do nothing.
 * Each of these looked fine in code review and wrong on screen.
 */
function renderCanvas(blocks: ThemeNode[], tokens: Record<string, unknown> = {}) {
  render(
    <FluidCanvas
      tokens={tokens}
      layout={{ shell: 'CenteredCard' }}
      blocks={blocks}
      assets={[]}
      selectedNodeId={null}
      showChrome={false}
      onSelectNode={vi.fn()}
    />,
  )
}

describe('FluidCanvas text nodes', () => {
  it('applies an explicit font size instead of the heading default', () => {
    renderCanvas([
      { id: 't1', type: 'Text', props: { text: 'Small label', font_size: '12px' } },
    ])

    const paragraph = screen.getByText('Small label')
    // The heading default must not win over the node's own size.
    expect(paragraph).not.toHaveClass('text-lg')
    expect(paragraph.parentElement?.parentElement).toHaveStyle({ fontSize: '12px' })
  })

  it('applies an explicit font weight instead of the heading default', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: { text: 'Light', font_weight: '400' } }])
    expect(screen.getByText('Light')).not.toHaveClass('font-semibold')
  })

  it('keeps the heading defaults when the node sets neither', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: { text: 'Welcome back' } }])

    const paragraph = screen.getByText('Welcome back')
    expect(paragraph).toHaveClass('text-lg')
    expect(paragraph).toHaveClass('font-semibold')
  })
})

/** The Box node's own element, not the shell card (which also sets a radius). */
function renderedBox() {
  return document.querySelector<HTMLElement>('.flex.w-full.flex-col[style*="border-radius"]')
}

function renderedRow() {
  return document.querySelector<HTMLElement>('.flex.w-full.flex-row[style*="border-radius"]')
}

describe('FluidCanvas context-bound text', () => {
  it('shows the binding when a node has only a text_path', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: { text_path: 'message' } }])

    // The builder cannot resolve context, but falling back to "Headline" made
    // every bound heading look like an unconfigured node.
    expect(screen.getByText('{message}')).toBeInTheDocument()
    expect(screen.queryByText('Headline')).not.toBeInTheDocument()
  })

  it('marks a binding as distinct from literal copy', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: { text_path: 'message' } }])

    const bound = screen.getByText('{message}')
    expect(bound).toHaveClass('italic')
    expect(bound).toHaveAttribute('title', 'Bound to context: message')
  })

  it('prefers a literal text over the binding as the design-time preview', () => {
    renderCanvas([
      { id: 't1', type: 'Text', props: { text: 'Something went wrong', text_path: 'message' } },
    ])

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    expect(screen.queryByText('{message}')).not.toBeInTheDocument()
  })

  it('still falls back to Headline with neither text nor binding', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: {} }])
    expect(screen.getByText('Headline')).toBeInTheDocument()
  })
})

describe('FluidCanvas boxes', () => {
  it('adds px to a unitless radius so the corners actually round', () => {
    renderCanvas([
      { id: 'b1', type: 'Box', props: { radius: 12, border_color: '#fff' }, children: [] },
    ])

    expect(renderedBox()).toHaveStyle({ borderRadius: '12px' })
  })

  it('passes through a radius that already has a unit', () => {
    renderCanvas([
      { id: 'b1', type: 'Box', props: { radius: '50%', border_color: '#fff' }, children: [] },
    ])

    expect(renderedBox()).toHaveStyle({ borderRadius: '50%' })
  })

  it('applies main-axis distribution from the layout', () => {
    renderCanvas([
      {
        id: 'b1',
        type: 'Box',
        layout: { direction: 'row', justify: 'center', align: 'center' },
        props: { border_color: '#fff', radius: 4 },
        children: [],
      },
    ])

    expect(renderedRow()).toHaveStyle({ justifyContent: 'center', alignItems: 'center' })
  })

  it('leaves distribution unset when the layout does not ask for it', () => {
    renderCanvas([
      {
        id: 'b1',
        type: 'Box',
        layout: { direction: 'row' },
        props: { border_color: '#fff', radius: 4 },
        children: [],
      },
    ])

    expect(renderedRow()?.style.justifyContent).toBe('')
  })
})

describe('FluidCanvas node spacing', () => {
  it('does not write zero margins, so container rhythm survives', () => {
    renderCanvas([{ id: 't1', type: 'Text', props: { text: 'Heading' } }])

    // An inline `margin-top: 0px` here silently defeated the form's space-y-*.
    const wrapper = screen.getByText('Heading').parentElement?.parentElement
    expect(wrapper?.style.marginTop).toBe('')
    expect(wrapper?.style.marginBottom).toBe('')
    expect(wrapper?.style.padding).toBe('')
  })

  it('still applies margins the node explicitly sets', () => {
    renderCanvas([
      {
        id: 't1',
        type: 'Text',
        props: { text: 'Spaced', margin_top: 24, margin_bottom: 8, padding: 4 },
      },
    ])

    const wrapper = screen.getByText('Spaced').parentElement?.parentElement
    expect(wrapper).toHaveStyle({
      marginTop: '24px',
      marginBottom: '8px',
      padding: '4px',
    })
  })
})

describe('FluidCanvas buttons', () => {
  it('uses a dark label on a light primary colour', () => {
    renderCanvas(
      [
        {
          id: 'btn',
          type: 'Component',
          component: 'Button',
          props: { label: 'Continue', variant: 'primary' },
        },
      ],
      { colors: { primary: '#ffffff' } },
    )

    // A hard-coded white label here was invisible.
    expect(screen.getByRole('button', { name: 'Continue' })).toHaveStyle({
      color: '#111827',
    })
  })

  it('uses a light label on a dark primary colour', () => {
    renderCanvas(
      [
        {
          id: 'btn',
          type: 'Component',
          component: 'Button',
          props: { label: 'Continue', variant: 'primary' },
        },
      ],
      { colors: { primary: '#111827' } },
    )

    expect(screen.getByRole('button', { name: 'Continue' })).toHaveStyle({
      color: '#ffffff',
    })
  })
})

describe('FluidCanvas links', () => {
  it('puts the alignment on the block wrapper, not the inline anchor', () => {
    renderCanvas([
      {
        id: 'l1',
        type: 'Component',
        component: 'Link',
        props: { label: 'Forgot password?', href: '/forgot', align: 'right' },
      },
    ])

    const anchor = screen.getByText('Forgot password?')
    // text-right on an inline element does nothing, so it must sit on an ancestor.
    expect(anchor).not.toHaveClass('text-right')
    expect(anchor.closest('.text-right')).not.toBeNull()
  })
})
