import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Switch } from './switch'

describe('Switch', () => {
  it('renders correctly', () => {
    render(<Switch aria-label="test-switch" />)
    expect(screen.getByRole('switch')).toBeInTheDocument()
  })

  it('can be toggled', async () => {
    render(<Switch aria-label="test-switch" />)
    const toggle = screen.getByRole('switch')
    
    expect(toggle).toHaveAttribute('data-state', 'unchecked')
    
    fireEvent.click(toggle)
    expect(toggle).toHaveAttribute('data-state', 'checked')
    
    fireEvent.click(toggle)
    expect(toggle).toHaveAttribute('data-state', 'unchecked')
  })

  it('is disabled when disabled prop is true', () => {
    render(<Switch disabled aria-label="test-switch" />)
    expect(screen.getByRole('switch')).toBeDisabled()
  })
})
