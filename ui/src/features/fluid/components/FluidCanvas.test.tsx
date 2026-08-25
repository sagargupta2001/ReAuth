import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidCanvas } from './FluidCanvas'
import type { ThemeNode } from '@/entities/theme/model/types'
import type { ProviderPreview } from '@/features/fluid/model/providerPreview'

const providerButtonsNode: ThemeNode = {
  id: 'node-providers',
  type: 'Component',
  component: 'ProviderButtons',
  size: { width: 'fill', height: 'hug' },
  props: { visible_if: 'enabled_providers_count' },
}

function renderCanvas(blocks: ThemeNode[], providers: ProviderPreview[] = []) {
  render(
    <FluidCanvas
      tokens={{ colors: { primary: '#111827' } }}
      layout={{ shell: 'CenteredCard' }}
      blocks={blocks}
      assets={[]}
      selectedNodeId={null}
      providers={providers}
      onSelectNode={vi.fn()}
    />,
  )
}

describe('FluidCanvas', () => {
  it('renders a button per enabled provider for ProviderButtons', () => {
    renderCanvas([providerButtonsNode], [
      { alias: 'google', display_name: 'Google', button_color: '#4285F4' },
      { alias: 'okta', display_name: 'Okta' },
    ])

    expect(screen.getByRole('button', { name: 'Google' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Okta' })).toBeInTheDocument()
    expect(screen.queryByText(/Unknown component/)).not.toBeInTheDocument()
  })

  it('keeps ProviderButtons visible with a placeholder when no providers exist', () => {
    renderCanvas([providerButtonsNode])

    expect(screen.getByText('No sign-in providers enabled for this realm.')).toBeInTheDocument()
    expect(screen.queryByText(/Unknown component/)).not.toBeInTheDocument()
  })

  it('still reports genuinely unknown components', () => {
    renderCanvas([
      { id: 'node-x', type: 'Component', component: 'NotARealComponent' },
    ])

    expect(screen.getByText('Unknown component: NotARealComponent')).toBeInTheDocument()
  })
})

describe('FluidCanvas new blocks', () => {
  it('previews a Checkbox with its label, inert', () => {
    renderCanvas([
      {
        id: 'cb',
        type: 'Component',
        component: 'Checkbox',
        props: { label: 'Remember me', name: 'remember_me' },
      },
    ])

    const box = screen.getByRole('checkbox', { name: 'Remember me' })
    expect(box).toBeInTheDocument()
    // The builder preview never accepts input.
    expect(box).toBeDisabled()
  })

  it('renders a Columns preset as a row, not a column', () => {
    renderCanvas([
      {
        id: 'cols',
        type: 'Box',
        layout: { direction: 'row', gap: 12, align: 'center', justify: 'between' },
        children: [
          { id: 'a', type: 'Text', props: { text: 'Left' } },
          { id: 'b', type: 'Text', props: { text: 'Right' } },
        ],
      },
    ])

    const row = screen.getByText('Left').closest('.flex-row')
    expect(row).not.toBeNull()
    expect(row?.contains(screen.getByText('Right'))).toBe(true)
  })

  it('renders a Heading preset larger than a plain Text default', () => {
    renderCanvas([
      {
        id: 'h',
        type: 'Text',
        props: { text: 'Welcome back' },
        style: { typography: { size: '24px', weight: '700' } },
      },
    ])

    const heading = screen.getByText('Welcome back')
    // The node sets its own size, so the heading default must not win.
    expect(heading).not.toHaveClass('text-lg')
    expect(heading.parentElement?.parentElement).toHaveStyle({ fontSize: '24px' })
  })
})
