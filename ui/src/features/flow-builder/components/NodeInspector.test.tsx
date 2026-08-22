import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('@/features/theme/api/useActiveTheme', () => ({
  useActiveTheme: () => ({ data: undefined }),
}))

import { useFlowBuilderStore } from '../store/flowBuilderStore'
import type { NodeContract } from '../store/flowBuilderStore'
import { NodeInspector } from './NodeInspector'

const contract = (id: string, supportsUi: boolean): NodeContract => ({
  id,
  category: supportsUi ? 'Authenticator' : 'Logic',
  display_name: id,
  description: '',
  icon: 'Box',
  inputs: [],
  outputs: [],
  config_schema: {},
  capabilities: { supports_ui: supportsUi },
})

const graphNode = (id: string, type: string) => ({
  id,
  type,
  position: { x: 0, y: 0 },
  data: { label: id },
})

describe('NodeInspector tabs', () => {
  beforeEach(() => {
    const store = useFlowBuilderStore.getState()
    store.reset()
    store.setNodeTypes([contract('ui.node', true), contract('logic.node', false)])
    store.setGraph(
      [
        graphNode('with-ui', 'ui.node'),
        graphNode('other-ui', 'ui.node'),
        graphNode('no-ui', 'logic.node'),
      ],
      [],
    )
  })

  const selectNode = (id: string) => {
    act(() => {
      useFlowBuilderStore.getState().selectNode(id)
    })
  }

  it('falls back to General when the new node has no Template tab', async () => {
    const user = userEvent.setup()
    selectNode('with-ui')
    render(<NodeInspector />)

    await user.click(screen.getByRole('tab', { name: 'Template' }))
    expect(await screen.findByText('Page Template')).toBeVisible()

    selectNode('no-ui')

    expect(screen.queryByRole('tab', { name: 'Template' })).not.toBeInTheDocument()
    expect(screen.getByText('Node Label')).toBeVisible()
  })

  it('resets the active tab when switching between nodes that share the tab', async () => {
    const user = userEvent.setup()
    selectNode('with-ui')
    render(<NodeInspector />)

    await user.click(screen.getByRole('tab', { name: 'Template' }))
    expect(await screen.findByText('Page Template')).toBeVisible()

    selectNode('other-ui')

    expect(screen.getByText('Node Label')).toBeVisible()
    expect(screen.queryByText('Page Template')).not.toBeInTheDocument()
  })

  it('keeps the selected tab while the same node stays selected', async () => {
    const user = userEvent.setup()
    selectNode('with-ui')
    render(<NodeInspector />)

    await user.click(screen.getByRole('tab', { name: 'Parameters' }))
    const params = await screen.findByText('No configurable parameters for this node.')
    expect(params).toBeVisible()

    act(() => {
      useFlowBuilderStore.getState().updateNodeData('with-ui', { label: 'Renamed' })
    })

    expect(screen.getByText('No configurable parameters for this node.')).toBeVisible()
  })
})
