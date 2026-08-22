import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom'
import { describe, expect, it } from 'vitest'
import userEvent from '@testing-library/user-event'

import { useRoutedTab } from './useRoutedTab'

const TABS = ['overview', 'history', 'settings'] as const

function Harness() {
  const { activeTab, onTabChange } = useRoutedTab({
    tabs: TABS,
    basePath: '/themes/theme-1',
  })
  const location = useLocation()

  return (
    <div>
      <span data-testid="active">{activeTab}</span>
      <span data-testid="path">{location.pathname}</span>
      {TABS.map((tab) => (
        <button key={tab} onClick={() => onTabChange(tab)}>
          go {tab}
        </button>
      ))}
    </div>
  )
}

function renderAt(path: string) {
  render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/:realm/themes/:themeId/:tab?" element={<Harness />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('useRoutedTab', () => {
  it('reads the active tab from the route', () => {
    renderAt('/master/themes/theme-1/history')
    expect(screen.getByTestId('active')).toHaveTextContent('history')
  })

  it('redirects to the first tab when none is in the URL', () => {
    renderAt('/master/themes/theme-1')

    expect(screen.getByTestId('active')).toHaveTextContent('overview')
    // The URL must describe what is shown, so it is linkable and bookmarkable.
    expect(screen.getByTestId('path')).toHaveTextContent('/master/themes/theme-1/overview')
  })

  it('canonicalises an unknown tab slug', () => {
    renderAt('/master/themes/theme-1/nonsense')

    expect(screen.getByTestId('active')).toHaveTextContent('overview')
    expect(screen.getByTestId('path')).toHaveTextContent('/master/themes/theme-1/overview')
  })

  it('navigates on tab change, keeping the realm prefix', async () => {
    renderAt('/master/themes/theme-1/overview')

    await userEvent.click(screen.getByRole('button', { name: 'go settings' }))

    expect(screen.getByTestId('path')).toHaveTextContent('/master/themes/theme-1/settings')
    expect(screen.getByTestId('active')).toHaveTextContent('settings')
  })

  it('preserves a non-default realm', () => {
    renderAt('/acme/themes/theme-1/settings')
    expect(screen.getByTestId('path')).toHaveTextContent('/acme/themes/theme-1/settings')
    expect(screen.getByTestId('active')).toHaveTextContent('settings')
  })
})
