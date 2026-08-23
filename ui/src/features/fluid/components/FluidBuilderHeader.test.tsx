import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidBuilderHeader } from './FluidBuilderHeader'
import type { ThemePageTemplate } from '@/entities/theme/model/types'

vi.mock('@/entities/realm/lib/navigation.logic', () => ({
  useRealmNavigate: () => vi.fn(),
}))

const PAGES: ThemePageTemplate[] = [
  {
    key: 'login',
    label: 'Login',
    description: 'Sign-in page',
    category: 'auth',
    blueprint: { layout: 'default', nodes: [] },
  },
]

function renderHeader(overrides: Partial<Parameters<typeof FluidBuilderHeader>[0]> = {}) {
  const props = {
    themeName: 'Default',
    pages: PAGES,
    activePageKey: 'login',
    onSelectPage: vi.fn(),
    onPublish: vi.fn(),
    ...overrides,
  }
  render(<FluidBuilderHeader {...props} />)
  return props
}

describe('FluidBuilderHeader restore-page action', () => {
  it('is absent when no reset handler is supplied', () => {
    renderHeader()
    expect(screen.queryByRole('button', { name: /restore page/i })).not.toBeInTheDocument()
  })

  it('is shown when a reset handler is supplied', () => {
    renderHeader({ onResetPage: vi.fn() })
    expect(screen.getByRole('button', { name: /restore page/i })).toBeInTheDocument()
  })

  it('confirms before resetting, naming the page', () => {
    const onResetPage = vi.fn()
    renderHeader({ onResetPage })

    fireEvent.click(screen.getByRole('button', { name: /restore page/i }))

    // Destructive and hard to notice, so it must not fire on the first click.
    expect(onResetPage).not.toHaveBeenCalled()
    expect(screen.getByText(/Restore the Login page to its default\?/i)).toBeInTheDocument()
  })

  it('resets once the confirmation is accepted', () => {
    const onResetPage = vi.fn()
    renderHeader({ onResetPage })

    fireEvent.click(screen.getByRole('button', { name: /restore page/i }))
    const dialog = screen.getByRole('alertdialog')
    fireEvent.click(
      [...dialog.querySelectorAll('button')].find(
        (button) => button.textContent?.trim() === 'Restore page',
      )!,
    )

    expect(onResetPage).toHaveBeenCalledTimes(1)
  })

  it('says the action does not touch theme-wide settings', () => {
    renderHeader({ onResetPage: vi.fn() })
    fireEvent.click(screen.getByRole('button', { name: /restore page/i }))

    // Users read "restore default" as theme-wide and expected colours to revert.
    expect(screen.getByText(/does not touch theme-wide settings/i)).toBeInTheDocument()
  })

  it('is disabled when the page has no default to restore', () => {
    renderHeader({ onResetPage: vi.fn(), canResetPage: false })
    expect(screen.getByRole('button', { name: /restore page/i })).toBeDisabled()
  })

  it('is disabled while a save or publish is in flight', () => {
    renderHeader({ onResetPage: vi.fn(), isPublishing: true })
    expect(screen.getByRole('button', { name: /restore page/i })).toBeDisabled()
  })
})
