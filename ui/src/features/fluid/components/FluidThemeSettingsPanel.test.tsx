import { fireEvent, render, screen } from '@testing-library/react'
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
