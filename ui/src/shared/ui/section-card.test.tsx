import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { SECTION_CARD_PANEL, SectionCard } from './section-card'

describe('SectionCard', () => {
  it('renders title, description, content and footer', () => {
    render(
      <SectionCard title="General Settings" description="Basic identity." footer="Footer note">
        <span>Body</span>
      </SectionCard>,
    )

    expect(screen.getByText('General Settings')).toBeInTheDocument()
    expect(screen.getByText('Basic identity.')).toBeInTheDocument()
    expect(screen.getByText('Body')).toBeInTheDocument()
    expect(screen.getByText('Footer note')).toBeInTheDocument()
  })

  it('omits description and footer when not given', () => {
    render(
      <SectionCard title="Bare">
        <span>Body</span>
      </SectionCard>,
    )

    expect(screen.getByText('Bare')).toBeInTheDocument()
    expect(screen.queryByText('Footer note')).not.toBeInTheDocument()
  })

  it('always wraps content in the inset panel', () => {
    render(
      <SectionCard title="T">
        <span>Body</span>
      </SectionCard>,
    )

    const panel = screen.getByText('Body').parentElement
    SECTION_CARD_PANEL.split(' ').forEach((className) => {
      expect(panel).toHaveClass(className)
    })
    expect(panel).toHaveClass('space-y-4')
  })

  it('lets the caller replace the panel layout but keeps its padding', () => {
    render(
      <SectionCard title="T" contentClassName="flex items-center justify-between gap-4">
        <span>Body</span>
      </SectionCard>,
    )

    const panel = screen.getByText('Body').parentElement
    expect(panel).toHaveClass('flex')
    expect(panel).toHaveClass('justify-between')
    expect(panel).not.toHaveClass('space-y-4')
    expect(panel).toHaveClass('p-4')
  })
})
