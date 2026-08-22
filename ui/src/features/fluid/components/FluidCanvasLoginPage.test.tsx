import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidCanvas } from './FluidCanvas'
import type { ThemeNode } from '@/entities/theme/model/types'

/**
 * Renders the seeded login blueprint from `theme_pages.rs::default_login_blueprint`.
 *
 * Kept in sync by hand: if the seed changes, this fixture must change with it.
 * It guards the parts of the page that only exist in blueprint data — the
 * placeholders and the centred sign-up row — which no other test covers.
 */
const SEEDED_LOGIN_NODES: ThemeNode[] = [
  { id: 'n0', type: 'Text', props: { text: 'Welcome back' } },
  {
    id: 'n1',
    type: 'Component',
    component: 'Input',
    props: {
      label: 'Email or username',
      name: 'username',
      input_type: 'text',
      placeholder: 'you@company.com',
    },
  },
  {
    id: 'n2',
    type: 'Component',
    component: 'Input',
    props: {
      label: 'Password',
      name: 'password',
      input_type: 'password',
      placeholder: 'Enter your password',
    },
  },
  {
    id: 'n3',
    type: 'Component',
    component: 'Link',
    props: { label: 'Forgot password?', href: '/forgot-password', align: 'right' },
  },
  {
    id: 'n4',
    type: 'Component',
    component: 'Button',
    props: { label: 'Continue', variant: 'primary' },
  },
  {
    id: 'n5',
    type: 'Component',
    component: 'ProviderButtons',
    props: { visible_if: 'enabled_providers_count' },
  },
  {
    id: 'n6',
    type: 'Box',
    size: { width: 'fill', height: 'hug' },
    layout: { direction: 'row', justify: 'center', align: 'baseline', gap: 4 },
    props: { visible_if: 'capabilities.registration_enabled' },
    children: [
      {
        id: 'n6a',
        type: 'Text',
        size: { width: 'hug', height: 'hug' },
        props: { text: 'New on our platform?', font_size: '12px', font_weight: '400' },
      },
      {
        id: 'n6b',
        type: 'Component',
        component: 'Link',
        size: { width: 'hug', height: 'hug' },
        props: { label: 'Create an account', href: '/register' },
      },
    ],
  },
]

function renderLoginPage() {
  render(
    <FluidCanvas
      tokens={{
        colors: {
          primary: 'var(--primary)',
          background: 'var(--background)',
          text: 'var(--foreground)',
          surface: 'var(--card)',
        },
        typography: { font_family: 'system-ui', base_size: 16 },
        radius: { base: 8 },
      }}
      layout={{ shell: 'CenteredCard', slots: ['main'] }}
      blocks={SEEDED_LOGIN_NODES}
      assets={[]}
      selectedNodeId={null}
      showChrome={false}
      onSelectNode={vi.fn()}
    />,
  )
}

describe('seeded login page', () => {
  it('shows a placeholder on both fields', () => {
    renderLoginPage()
    expect(screen.getByPlaceholderText('you@company.com')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Enter your password')).toBeInTheDocument()
  })

  it('puts the sign-up prompt and link on one centred row', () => {
    renderLoginPage()

    const prompt = screen.getByText('New on our platform?')
    const link = screen.getByText('Create an account')
    const row = prompt.closest('.flex-row')

    expect(row).not.toBeNull()
    // baseline, not center: the prompt and the link have different line
    // heights, so centring their boxes leaves the text misaligned.
    expect(row).toHaveStyle({ justifyContent: 'center', alignItems: 'baseline' })
    // Both halves live in the same row, so they read as one sentence.
    expect(row?.contains(link)).toBe(true)
  })

  it('links only the "Create an account" half', () => {
    renderLoginPage()

    expect(screen.getByText('Create an account').tagName).toBe('A')
    expect(screen.getByText('New on our platform?').closest('a')).toBeNull()
  })

  it('orders the page: fields, forgot link, Continue, then sign-up', () => {
    renderLoginPage()

    const order = ['Password', 'Forgot password?', 'Continue', 'New on our platform?'].map(
      (label) => {
        const node = screen.getByText(label)
        return node.compareDocumentPosition(screen.getByText('Welcome back'))
      },
    )
    // Every one of them follows the heading in document order.
    order.forEach((position) => {
      expect(position & Node.DOCUMENT_POSITION_PRECEDING).toBeTruthy()
    })

    const continueButton = screen.getByRole('button', { name: 'Continue' })
    const signupRow = screen.getByText('New on our platform?')
    expect(
      continueButton.compareDocumentPosition(signupRow) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('keeps the forgot-password link right-aligned', () => {
    renderLoginPage()
    expect(screen.getByText('Forgot password?').closest('.text-right')).not.toBeNull()
  })
})
