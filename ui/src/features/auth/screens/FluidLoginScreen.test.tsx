import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import type { ThemeNode } from '@/entities/theme/model/types'

const snapshot = vi.hoisted(() => ({ current: null as unknown }))
vi.mock('@/features/theme/api/useThemeSnapshot', () => ({
  useThemeSnapshot: () => ({ data: snapshot.current, isLoading: false }),
}))

const { FluidLoginScreen } = await import('./FluidLoginScreen')

/**
 * Covers the runtime `FluidHost` — the half of the renderer that is live rather
 * than inert. The shared walker is unit-tested against a stub host in
 * `renderFluidNode.test.tsx`; what needs asserting here is that the real page
 * still wires forms, gates visibility, and hides provider buttons.
 */

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

function renderScreen(nodes: ThemeNode[], context: Record<string, unknown> = {}) {
  snapshot.current = {
    tokens: TOKENS,
    layout: { shell: 'CenteredCard', slots: ['main'] },
    nodes,
    assets: [],
  }
  const onSubmit = vi.fn().mockResolvedValue(undefined)
  render(
    <FluidLoginScreen
      context={context}
      onSubmit={onSubmit}
      isLoading={false}
      error={null}
      realm="master"
    />,
  )
  return { onSubmit }
}

const usernameField: ThemeNode = {
  id: 'field-username',
  type: 'Component',
  component: 'Input',
  props: { label: 'Username', name: 'username', placeholder: 'you@example.com' },
}

const passwordField: ThemeNode = {
  id: 'field-password',
  type: 'Component',
  component: 'Input',
  props: {
    label: 'Password',
    name: 'password',
    input_type: 'password',
    placeholder: 'Your password',
  },
}

const submitButton: ThemeNode = {
  id: 'btn-submit',
  type: 'Component',
  component: 'Button',
  props: { label: 'Continue', variant: 'primary' },
}

describe('FluidLoginScreen runtime host', () => {
  it('wires an Input to the form and submits what was typed', async () => {
    const { onSubmit } = renderScreen([usernameField, passwordField, submitButton])

    fireEvent.change(screen.getByPlaceholderText('you@example.com'), {
      target: { value: 'ada@example.com' },
    })
    fireEvent.change(screen.getByPlaceholderText('Your password'), {
      target: { value: 'hunter2' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }))

    await waitFor(() => expect(onSubmit).toHaveBeenCalled())
    expect(onSubmit.mock.calls[0][0]).toMatchObject({
      username: 'ada@example.com',
      password: 'hunter2',
    })
  })

  it('surfaces a validation failure instead of submitting', async () => {
    const { onSubmit } = renderScreen([usernameField, passwordField, submitButton])

    fireEvent.change(screen.getByPlaceholderText('you@example.com'), {
      target: { value: 'ada@example.com' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }))

    expect(await screen.findByText(/Password must be at least 4 characters/)).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('submits on a Button with no click action, rather than needing one', () => {
    renderScreen([usernameField, submitButton])
    // A plain button is the form's submit; only an action-bound one is type=button.
    expect(screen.getByRole('button', { name: 'Continue' })).toHaveAttribute('type', 'submit')
  })

  it('hides a node the blueprint marks invisible', () => {
    renderScreen([
      { id: 'hidden', type: 'Text', props: { text: 'Secret', visible: false } },
      { id: 'shown', type: 'Text', props: { text: 'Public' } },
    ])
    expect(screen.queryByText('Secret')).not.toBeInTheDocument()
    expect(screen.getByText('Public')).toBeInTheDocument()
  })

  it('resolves a bound Text against the auth context', () => {
    renderScreen([{ id: 't', type: 'Text', props: { text_path: 'message' } }], {
      message: 'Check your inbox',
    })
    expect(screen.getByText('Check your inbox')).toBeInTheDocument()
  })

  it('drops the provider block entirely when the realm has none enabled', () => {
    renderScreen([
      { id: 'p', type: 'Component', component: 'ProviderButtons' },
      { id: 't', type: 'Text', props: { text: 'Only me' } },
    ])
    // The builder shows a placeholder here; the runtime must render nothing.
    expect(screen.queryByText(/sign-in providers/i)).not.toBeInTheDocument()
    expect(screen.getByText('Only me')).toBeInTheDocument()
  })

  it('renders a button per enabled provider', () => {
    renderScreen([{ id: 'p', type: 'Component', component: 'ProviderButtons' }], {
      enabled_providers: [
        { alias: 'google', display_name: 'Google' },
        { alias: 'github', display_name: 'GitHub' },
      ],
    })
    expect(screen.getByRole('button', { name: 'Google' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'GitHub' })).toBeInTheDocument()
  })

  it('renders a password field as a password input', () => {
    renderScreen([
      {
        id: 'pw',
        type: 'Component',
        component: 'Input',
        props: { label: 'Password', name: 'password', input_type: 'password', placeholder: 'Secret' },
      },
    ])
    expect(screen.getByPlaceholderText('Secret')).toHaveAttribute('type', 'password')
  })

  it('submits a Checkbox value with the form', async () => {
    const { onSubmit } = renderScreen([
      usernameField,
      passwordField,
      {
        id: 'cb',
        type: 'Component',
        component: 'Checkbox',
        props: { label: 'Remember me', name: 'remember_me' },
      },
      submitButton,
    ])

    fireEvent.change(screen.getByPlaceholderText('you@example.com'), {
      target: { value: 'ada@example.com' },
    })
    fireEvent.change(screen.getByPlaceholderText('Your password'), {
      target: { value: 'hunter2' },
    })
    fireEvent.click(screen.getByRole('checkbox', { name: 'Remember me' }))
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }))

    await waitFor(() => expect(onSubmit).toHaveBeenCalled())
    expect(onSubmit.mock.calls[0][0]).toMatchObject({ remember_me: true })
  })

  it('treats a checkbox left alone as false, not as its absence', () => {
    renderScreen([
      {
        id: 'cb',
        type: 'Component',
        component: 'Checkbox',
        props: { label: 'Marketing emails', name: 'opt_in' },
      },
    ])
    expect(screen.getByRole('checkbox', { name: 'Marketing emails' })).not.toBeChecked()
  })

  it('honours a blueprint default of checked', () => {
    renderScreen([
      {
        id: 'cb',
        type: 'Component',
        component: 'Checkbox',
        props: { label: 'Remember me', name: 'remember_me', checked: true },
      },
    ])
    expect(screen.getByRole('checkbox', { name: 'Remember me' })).toBeChecked()
  })

  it('reads the inspector select\'s string "false" as unchecked', () => {
    // The inspector writes strings, and `Boolean('false')` is true.
    renderScreen([
      {
        id: 'cb',
        type: 'Component',
        component: 'Checkbox',
        props: { label: 'Remember me', name: 'remember_me', checked: 'false' },
      },
    ])
    expect(screen.getByRole('checkbox', { name: 'Remember me' })).not.toBeChecked()
  })

  it('nests children inside a Box, in order', () => {
    renderScreen([
      {
        id: 'row',
        type: 'Box',
        layout: { direction: 'row', gap: 4 },
        children: [
          { id: 'a', type: 'Text', props: { text: 'New here?' } },
          { id: 'b', type: 'Component', component: 'Link', props: { label: 'Sign up', href: '/r' } },
        ],
      },
    ])
    const row = screen.getByText('New here?').closest('.flex-row')
    expect(row).not.toBeNull()
    expect(row?.contains(screen.getByText('Sign up'))).toBe(true)
  })
})
