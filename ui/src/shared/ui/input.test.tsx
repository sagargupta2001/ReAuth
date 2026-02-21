import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Input } from './input'

describe('Input', () => {
  it('renders correctly', () => {
    render(<Input placeholder="Test Input" />)
    expect(screen.getByPlaceholderText('Test Input')).toBeInTheDocument()
  })

  it('allows user input', () => {
    render(<Input placeholder="Test Input" />)
    const input = screen.getByPlaceholderText('Test Input') as HTMLInputElement
    fireEvent.change(input, { target: { value: 'New Value' } })
    expect(input.value).toBe('New Value')
  })

  it('is disabled when disabled prop is true', () => {
    render(<Input disabled placeholder="Test Input" />)
    expect(screen.getByPlaceholderText('Test Input')).toBeDisabled()
  })
})
