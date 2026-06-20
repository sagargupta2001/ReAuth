import { act, render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, it, expect } from 'vitest'

import { useBreadcrumbStore } from '../model/useBreadcrumbStore'
import { useBreadcrumbTrail } from './useBreadcrumbTrail'

function Probe() {
  const trail = useBreadcrumbTrail()
  return (
    <ul>
      {trail.map((n) => (
        <li
          key={n.id}
          data-current={n.isCurrent ? 'true' : 'false'}
          data-href={n.href ?? ''}
          data-siblings={(n.siblings ?? []).map((s) => s.href).join(',')}
        >
          {typeof n.label === 'string' ? n.label : ''}
        </li>
      ))}
    </ul>
  )
}

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/:realm/*" element={<Probe />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('useBreadcrumbTrail', () => {
  it('shows a single current node for top-level listing routes (acts as the title)', () => {
    renderAt('/master/users')
    const items = screen.getAllByRole('listitem')
    expect(items.map((i) => i.textContent)).toEqual(['Users'])
    expect(items[0]).toHaveAttribute('data-current', 'true')
  })

  it('resolves the realm root to a single "Overview" node', () => {
    render(
      <MemoryRouter initialEntries={['/master']}>
        <Routes>
          <Route path="/:realm" element={<Probe />} />
        </Routes>
      </MemoryRouter>,
    )
    expect(screen.getAllByRole('listitem').map((i) => i.textContent)).toEqual(['Overview'])
  })

  it('builds a navigable trail for a detail route, omitting the realm', () => {
    renderAt('/master/users/u-123/sessions')
    const items = screen.getAllByRole('listitem')
    expect(items.map((i) => i.textContent)).toEqual(['Users', 'User', 'Sessions'])

    // Section links, current page does not.
    expect(items[0]).toHaveAttribute('data-href', '/master/users')
    expect(items[2]).toHaveAttribute('data-current', 'true')
    expect(items[2]).toHaveAttribute('data-href', '')
  })

  it('renders an active tab as a quick-switch node with all sibling tabs', () => {
    renderAt('/master/users/u-123/roles')
    const items = screen.getAllByRole('listitem')
    expect(items.map((i) => i.textContent)).toEqual(['Users', 'User', 'Roles'])

    const tabNode = items[2]
    expect(tabNode).toHaveAttribute('data-current', 'true')
    // The "roles" tab must NOT borrow the top-level Roles section link/registry.
    expect(tabNode).toHaveAttribute('data-href', '')
    expect(tabNode.getAttribute('data-siblings')?.split(',')).toEqual([
      '/master/users/u-123/profile',
      '/master/users/u-123/roles',
      '/master/users/u-123/credentials',
      '/master/users/u-123/settings',
    ])
  })

  it('skips the structural "webhooks" passthrough segment', () => {
    renderAt('/master/events/webhooks/t-9/deliveries')
    expect(screen.getAllByRole('listitem').map((i) => i.textContent)).toEqual([
      'Webhooks',
      'Webhook',
      'Deliveries',
    ])
  })

  it('applies store overrides for dynamic id segments', () => {
    act(() => useBreadcrumbStore.setState({ overrides: { 'u-123': 'Ada Lovelace' } }))
    renderAt('/master/users/u-123')
    expect(screen.getAllByRole('listitem').map((i) => i.textContent)).toEqual([
      'Users',
      'Ada Lovelace',
    ])
    act(() => useBreadcrumbStore.setState({ overrides: {} }))
  })

  it('renders Settings as a non-link section with a current sub-page', () => {
    renderAt('/master/settings/email')
    const items = screen.getAllByRole('listitem')
    expect(items.map((i) => i.textContent)).toEqual(['Settings', 'Email'])
    expect(items[0]).toHaveAttribute('data-href', '') // no index page
  })
})
