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
