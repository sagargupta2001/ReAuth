import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Checkbox } from './checkbox'

describe('Checkbox', () => {
  it('renders correctly', () => {
    render(<Checkbox aria-label="test-checkbox" />)
    expect(screen.getByRole('checkbox')).toBeInTheDocument()
  })

  it('can be checked and unchecked', async () => {
    render(<Checkbox aria-label="test-checkbox" />)
    const checkbox = screen.getByRole('checkbox')
    
    expect(checkbox).toHaveAttribute('data-state', 'unchecked')
    
    fireEvent.click(checkbox)
    expect(checkbox).toHaveAttribute('data-state', 'checked')
    
    fireEvent.click(checkbox)
    expect(checkbox).toHaveAttribute('data-state', 'unchecked')
  })

  it('is disabled when disabled prop is true', () => {
    render(<Checkbox disabled aria-label="test-checkbox" />)
    expect(screen.getByRole('checkbox')).toBeDisabled()
  })
})
