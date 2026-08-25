import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import type { ThemeNode } from '@/entities/theme/model/types'
import { renderFluidNode, type FluidHost } from './renderFluidNode'

/**
 * A host that records what the walker asked it to wrap, so a test can assert on
 * the wrapper classes each branch produces without rendering a whole page.
 */
function recordingHost(overrides: Partial<FluidHost> = {}) {
  const wrapped: Array<{ nodeId: string; className: string }> = []
  const host: FluidHost = {
    primary: '#3355ff',
    componentTheme: { text: '#111111', radius: 8 },
    assets: new Map(),
    isVisible: () => true,
    wrap: ({ node, content, className }) => {
      wrapped.push({ nodeId: node.id, className: className ?? '' })
      return (
        <div key={node.id} data-node={node.id} className={className}>
          {content}
        </div>
      )
    },
    renderText: (_node, visuals) => <p>{String(visuals.props.text ?? '')}</p>,
    renderInput: (_node, spec) => <input placeholder={spec.placeholder} readOnly />,
    renderButton: (_node, spec) => <button type="button">{spec.defaultLabel}</button>,
    renderProviders: () => <div>providers</div>,
    ...overrides,
  }
  const classFor = (nodeId: string) =>
    wrapped.find((entry) => entry.nodeId === nodeId)?.className ?? ''
  return { host, wrapped, classFor }
}

const PROBE = 'probe-wrapper-class'

/** One node of every type the walker branches on. */
const NODES: ThemeNode[] = [
  { id: 'n-box', type: 'Box', children: [] },
  { id: 'n-text', type: 'Text', props: { text: 'Hello' } },
  { id: 'n-icon', type: 'Icon', props: { name: 'mail' } },
  { id: 'n-input', type: 'Input', props: { name: 'email' } },
  { id: 'n-image', type: 'Image', props: { asset_id: '' } },
  { id: 'n-button', type: 'Component', component: 'Button', props: { label: 'Go' } },
  { id: 'n-link', type: 'Component', component: 'Link', props: { label: 'Here' } },
  { id: 'n-divider', type: 'Component', component: 'Divider' },
  { id: 'n-providers', type: 'Component', component: 'ProviderButtons' },
]

describe('renderFluidNode wrapper classes', () => {
  it.each(NODES.map((node) => [node.id, node] as const))(
    'passes the caller wrapper class through for %s',
    (_id, node) => {
      // Four branches used to merge `options.wrapperClass` by hand and forget,
      // so brand-slot text was white at runtime but not in the builder. `wrap`
      // merges it centrally now; this asserts no branch can bypass that.
      const { host, classFor } = recordingHost()
      render(<div>{renderFluidNode(node, host, 0, { wrapperClass: PROBE })}</div>)
      expect(classFor(node.id)).toContain(PROBE)
    },
  )
})

describe('renderFluidNode branch behaviour', () => {
  it('renders nothing when the host hides the node', () => {
    const { host, wrapped } = recordingHost({ isVisible: () => false })
    const { container } = render(
      <div>{renderFluidNode(NODES[1], host, 0)}</div>,
    )
    expect(wrapped).toEqual([])
    expect(container.textContent).toBe('')
  })

  it('hides a provider block entirely when the host returns null', () => {
    // The runtime drops the block when the realm has no providers; the builder
    // returns a placeholder instead so the node stays selectable.
    const { host, wrapped } = recordingHost({ renderProviders: () => null })
    render(<div>{renderFluidNode(NODES[8], host, 0)}</div>)
    expect(wrapped).toEqual([])
  })

  it('gives a Divider no padding of its own', () => {
    // The builder used to add `py-2` that the runtime did not, so the preview
    // spaced dividers differently from the real page.
    const { host, classFor } = recordingHost()
    render(<div>{renderFluidNode(NODES[7], host, 0)}</div>)
    expect(classFor('n-divider')).not.toContain('py-2')
  })

  it('applies text alignment to an Image wrapper', () => {
    // Alignment worked in the builder and silently did nothing at runtime.
    const { host, classFor } = recordingHost()
    const image: ThemeNode = { id: 'n-img', type: 'Image', props: { align: 'right' } }
    render(<div>{renderFluidNode(image, host, 0)}</div>)
    expect(classFor('n-img')).toContain('text-right')
  })

  it('recurses into Box children', () => {
    const { host, wrapped } = recordingHost()
    const tree: ThemeNode = {
      id: 'outer',
      type: 'Box',
      children: [{ id: 'inner', type: 'Text', props: { text: 'Nested' } }],
    }
    render(<div>{renderFluidNode(tree, host, 0)}</div>)
    expect(wrapped.map((entry) => entry.nodeId)).toEqual(['inner', 'outer'])
    expect(screen.getByText('Nested')).toBeInTheDocument()
  })

  it('expands a Component through the registry before branching on its name', () => {
    const { host, wrapped } = recordingHost()
    const input: ThemeNode = {
      id: 'n-field',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email', name: 'email', placeholder: 'you@example.com' },
    }
    render(<div>{renderFluidNode(input, host, 0)}</div>)
    // The expansion produces the label, field container, and inner input, none
    // of which are authored nodes.
    expect(wrapped.length).toBeGreaterThan(1)
    expect(screen.getByPlaceholderText('you@example.com')).toBeInTheDocument()
  })

  it('marks a Component expansion as non-selectable for the host', () => {
    const seen: Array<boolean | undefined> = []
    const { host } = recordingHost({
      wrap: ({ node, content, options }) => {
        seen.push(options?.disableSelection)
        return <div key={node.id}>{content}</div>
      },
    })
    const input: ThemeNode = {
      id: 'n-field',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email', name: 'email' },
    }
    render(<div>{renderFluidNode(input, host, 0)}</div>)
    // The authored node stays selectable; everything the registry generated
    // beneath it does not.
    expect(seen.filter((value) => value === true).length).toBeGreaterThan(0)
  })

  it('asks the host for the anchor props a Link needs', () => {
    const linkProps = vi.fn(() => ({ onClick: vi.fn() }))
    const { host } = recordingHost({ linkProps })
    render(<div>{renderFluidNode(NODES[6], host, 0)}</div>)
    expect(linkProps).toHaveBeenCalledWith(NODES[6])
    expect(screen.getByText('Here').tagName).toBe('A')
  })

  it('reports an unknown component rather than rendering nothing', () => {
    const { host } = recordingHost()
    const node: ThemeNode = { id: 'n-x', type: 'Component', component: 'Mystery' }
    render(<div>{renderFluidNode(node, host, 0)}</div>)
    expect(screen.getByText(/Unknown component: Mystery/)).toBeInTheDocument()
  })
})
