import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidCanvas } from './FluidCanvas'
import type { ThemeNode } from '@/entities/theme/model/types'

const snapshot = vi.hoisted(() => ({ current: null as unknown }))
vi.mock('@/features/theme/api/useThemeSnapshot', () => ({
  useThemeSnapshot: () => ({ data: snapshot.current, isLoading: false }),
}))

const { FluidLoginScreen } = await import('@/features/auth/screens/FluidLoginScreen')

const TOKENS = {
  colors: {
    primary: 'var(--primary)',
    background: 'var(--background)',
    text: 'var(--foreground)',
    surface: 'var(--card)',
  },
  typography: { font_family: 'system-ui', base_size: 16 },
  radius: { base: 8 },
}

const LAYOUT = { shell: 'CenteredCard', slots: ['main'] }

/**
 * Three authored levels: outer column > inner row > text.
 *
 * Distinct gaps per level are what let the assertion name a level without
 * depending on how each renderer wraps its nodes — the builder adds a
 * selection wrapper the runtime does not have.
 */
const NESTED_NODES: ThemeNode[] = [
  {
    id: 'outer',
    type: 'Box',
    size: { width: 'fill', height: 'hug' },
    layout: { direction: 'column', gap: 4, align: 'stretch' },
    children: [
      {
        id: 'inner',
        type: 'Box',
        size: { width: 'fill', height: 'hug' },
        layout: { direction: 'row', gap: 8, justify: 'center', align: 'baseline' },
        children: [
          {
            id: 'deep',
            type: 'Text',
            size: { width: 'hug', height: 'hug' },
            props: { text: 'Deepest', font_size: '12px' },
          },
        ],
      },
    ],
  },
]

/** Flex boxes between the deepest text and the document root, innermost first. */
function boxChain(): HTMLElement[] {
  const chain: HTMLElement[] = []
  let element = screen.getByText('Deepest').parentElement
  while (element) {
    if (element.classList.contains('flex') && element.style.gap) {
      chain.push(element)
    }
    element = element.parentElement
  }
  return chain
}

function describeChain() {
  return boxChain().map((element) => ({
    gap: element.style.gap,
    direction: element.classList.contains('flex-row') ? 'row' : 'column',
    alignItems: element.style.alignItems,
    justifyContent: element.style.justifyContent,
  }))
}

describe('renderer parity for nested trees', () => {
  it('nests three levels identically in the builder canvas and at runtime', () => {
    const { unmount } = render(
      <FluidCanvas
        tokens={TOKENS}
        layout={LAYOUT}
        blocks={NESTED_NODES}
        assets={[]}
        selectedNodeId={null}
        showChrome={false}
        onSelectNode={vi.fn()}
      />,
    )
    const canvasChain = describeChain()
    unmount()

    snapshot.current = { tokens: TOKENS, layout: LAYOUT, nodes: NESTED_NODES, assets: [] }
    render(
      <FluidLoginScreen
        context={{}}
        onSubmit={vi.fn()}
        isLoading={false}
        error={null}
        realm="master"
      />,
    )
    const runtimeChain = describeChain()

    // Both renderers must place the text inside the row inside the column, with
    // the same layout derivation at every level. They are separate components
    // and have shipped divergent before.
    expect(canvasChain).toEqual([
      { gap: '8px', direction: 'row', alignItems: 'baseline', justifyContent: 'center' },
      { gap: '4px', direction: 'column', alignItems: 'stretch', justifyContent: '' },
    ])
    expect(runtimeChain).toEqual(canvasChain)
  })
})
