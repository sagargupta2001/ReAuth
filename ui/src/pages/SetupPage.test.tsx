import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { SetupPage } from './SetupPage'
import { SECTION_CARD_PANEL } from '@/shared/ui/section-card'

vi.mock('react-router-dom', async (importOriginal) => ({
  ...(await importOriginal<typeof import('react-router-dom')>()),
  useNavigate: () => vi.fn(),
}))

vi.mock('@/features/setup/api/useSetupStatus', () => ({
  useSetupStatus: () => ({
    data: { required: true },
    isLoading: false,
    isError: false,
    error: null,
  }),
}))

vi.mock('@/features/setup/api/useSetupBootstrap', () => ({
  useSetupBootstrap: () => ({ mutate: vi.fn(), isPending: false }),
}))

describe('SetupPage', () => {
  it('leads with the ReAuth logo', () => {
    render(<SetupPage />)

    const logo = screen.getByAltText('logo')
    expect(logo).toHaveAttribute('src', '/reauth.svg')
    expect(logo).toHaveClass('h-14')
    expect(logo).toHaveClass('w-14')
  })

  it('centres the title and description', () => {
    render(<SetupPage />)

    const header = screen.getByText('Initialize ReAuth').parentElement
    expect(header).toHaveClass('items-center')
    expect(header).toHaveClass('text-center')
    // The logo sits inside the same centred header, above the title.
    expect(header).toContainElement(screen.getByAltText('logo'))
  })

  it('does not give the form its own background panel', () => {
    render(<SetupPage />)

    const field = screen.getByLabelText('Setup token')
    const content = field.closest('div')?.parentElement
    expect(content).not.toBeNull()
    // A hero card is a single surface — no inset panel like the settings cards.
    SECTION_CARD_PANEL.split(' ').forEach((className) => {
      expect(content).not.toHaveClass(className)
    })
  })

  it('pads the content itself, since there is no inset panel to do it', () => {
    render(<SetupPage />)

    const content = screen.getByLabelText('Setup token').closest('div')?.parentElement
    expect(content).toHaveClass('px-6')
  })

  it('renders all three setup fields and the submit action', () => {
    render(<SetupPage />)

    expect(screen.getByLabelText('Setup token')).toBeInTheDocument()
    expect(screen.getByLabelText('Admin username')).toBeInTheDocument()
    expect(screen.getByLabelText('Admin password')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Create master admin' })).toBeDisabled()
  })

  it('keeps the decorative glow out of the accessibility tree', () => {
    const { container } = render(<SetupPage />)

    const glow = container.querySelector('.pulse-glow')
    expect(glow).toHaveAttribute('aria-hidden', 'true')
    expect(glow).toHaveClass('pointer-events-none')
  })
})
