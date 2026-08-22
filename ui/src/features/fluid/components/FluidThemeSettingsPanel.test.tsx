import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { FluidThemeSettingsPanel } from './FluidThemeSettingsPanel'
import type { ThemeAsset } from '@/entities/theme/model/types'
import { LayoutShell } from '@/features/fluid/model/layoutShells'
import { THEME_SETTINGS_SECTIONS } from '@/features/fluid/model/themeSettingsSchema'

function renderPanel(overrides: Partial<Parameters<typeof FluidThemeSettingsPanel>[0]> = {}) {
  const props = {
    tokens: { colors: { primary: '#111827' }, typography: { font_family: 'Inter' } },
    onTokensChange: vi.fn(),
    layout: { shell: LayoutShell.CenteredCard },
    onLayoutChange: vi.fn(),
    assets: [] as ThemeAsset[],
    onUploadAsset: vi.fn(),
    ...overrides,
  }
  render(<FluidThemeSettingsPanel {...props} />)
  return props
}

/** Scopes queries to one colour field's Token/Custom toggle. */
function sourceToggle(label: string) {
  return within(screen.getByRole('group', { name: `${label} source` }))
}

describe('FluidThemeSettingsPanel', () => {
  it('renders every section in the schema', () => {
    renderPanel()
    THEME_SETTINGS_SECTIONS.forEach((section) => {
      expect(screen.getByText(section.title)).toBeInTheDocument()
    })
  })

  it('writes a colour change to the right token path without dropping siblings', () => {
    const { onTokensChange } = renderPanel({
      tokens: { colors: { primary: '#111827', background: '#ffffff' } },
    })

    fireEvent.change(screen.getByLabelText('Primary'), { target: { value: '#ff0000' } })

    expect(onTokensChange).toHaveBeenCalledWith({
      colors: { primary: '#ff0000', background: '#ffffff' },
    })
  })

  it('shows a design-token picker instead of a swatch for var() colours', () => {
    renderPanel({ tokens: { colors: { primary: 'var(--primary)' } } })

    // No literal colour input while the token is a design-token reference.
    expect(screen.queryByLabelText('Primary color')).not.toBeInTheDocument()
    expect(sourceToggle('Primary').getByRole('button', { name: 'Design token' })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
  })

  it('converts a design token to the literal colour it resolves to', () => {
    const { onTokensChange } = renderPanel({
      tokens: { colors: { primary: 'var(--nope, #abcdef)' } },
    })

    fireEvent.click(sourceToggle('Primary').getByRole('button', { name: 'Custom' }))

    expect(onTokensChange).toHaveBeenCalledWith({ colors: { primary: '#abcdef' } })
  })

  it('switches a literal colour back to a design token', () => {
    const { onTokensChange } = renderPanel({ tokens: { colors: { primary: '#111827' } } })

    fireEvent.click(sourceToggle('Primary').getByRole('button', { name: 'Design token' }))

    expect(onTokensChange).toHaveBeenCalledWith({ colors: { primary: 'var(--primary)' } })
  })

  it('reports failing contrast pairs', () => {
    renderPanel({
      tokens: { colors: { text: '#777777', background: '#808080', surface: '#808080' } },
    })

    expect(screen.getByText(/below the WCAG AA minimum/)).toBeInTheDocument()
  })

  it('does not warn when contrast passes', () => {
    renderPanel({
      tokens: {
        colors: {
          text: '#000000',
          background: '#ffffff',
          surface: '#ffffff',
          primary: '#1d4ed8',
        },
      },
    })

    expect(screen.queryByText(/below the WCAG AA minimum/)).not.toBeInTheDocument()
  })

  it('keeps unrelated token groups when editing typography', () => {
    const { onTokensChange } = renderPanel({
      tokens: { colors: { primary: '#111827' }, typography: { font_family: 'Inter' } },
    })

    fireEvent.change(screen.getByLabelText('Font Family'), { target: { value: 'Roboto' } })

    expect(onTokensChange).toHaveBeenCalledWith({
      colors: { primary: '#111827' },
      typography: { font_family: 'Roboto' },
    })
  })

  it('normalizes slots when the layout shell changes', () => {
    const { onLayoutChange } = renderPanel({ layout: {} })

    fireEvent.click(screen.getByText('Split Screen'))

    expect(onLayoutChange).toHaveBeenCalledWith({
      shell: LayoutShell.SplitScreen,
      slots: ['main'],
    })
  })

  it('shows an empty state when there are no assets', () => {
    renderPanel()
    expect(screen.getByText('No assets uploaded yet.')).toBeInTheDocument()
  })

  it('lists uploaded assets with their size', () => {
    renderPanel({
      assets: [
        {
          id: 'asset-1',
          theme_id: 'theme-1',
          asset_type: 'image',
          filename: 'logo.png',
          mime_type: 'image/png',
          byte_size: 2048,
          created_at: '2026-01-01T00:00:00Z',
          updated_at: '2026-01-01T00:00:00Z',
          url: '/assets/logo.png',
        },
      ],
    })

    expect(screen.getByText('logo.png')).toBeInTheDocument()
    expect(screen.getByText('2.0 KB · image')).toBeInTheDocument()
  })

  it('reports upload progress on the upload button', () => {
    renderPanel({ isUploading: true })
    expect(screen.getByText('Uploading...')).toBeInTheDocument()
  })
})
