import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'

import { DynamicIsland } from './dynamic-island'

describe('DynamicIsland', () => {
  it('renders its content and exposes the slot + aria label', () => {
    render(
      <DynamicIsland contentKey="a" ariaLabel="location">
        <span>Hello</span>
      </DynamicIsland>,
    )

    expect(screen.getByText('Hello')).toBeInTheDocument()
    expect(screen.getByLabelText('location')).toHaveAttribute('data-slot', 'dynamic-island')
  })

  it('updates content when the key changes', () => {
    const { rerender } = render(
      <DynamicIsland contentKey="a">
        <span>One</span>
      </DynamicIsland>,
    )
    expect(screen.getByText('One')).toBeInTheDocument()

    rerender(
      <DynamicIsland contentKey="b">
        <span>Two</span>
      </DynamicIsland>,
    )
    expect(screen.getByText('Two')).toBeInTheDocument()
  })
})
