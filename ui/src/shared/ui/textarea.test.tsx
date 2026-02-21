import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Textarea } from './textarea'

describe('Textarea', () => {
  it('renders correctly', () => {
    render(<Textarea placeholder="Test Textarea" />)
    expect(screen.getByPlaceholderText('Test Textarea')).toBeInTheDocument()
  })

  it('allows user input', () => {
    render(<Textarea placeholder="Test Textarea" />)
    const textarea = screen.getByPlaceholderText('Test Textarea') as HTMLTextAreaElement
    fireEvent.change(textarea, { target: { value: 'New Value' } })
    expect(textarea.value).toBe('New Value')
  })

  it('is disabled when disabled prop is true', () => {
    render(<Textarea disabled placeholder="Test Textarea" />)
    expect(screen.getByPlaceholderText('Test Textarea')).toBeDisabled()
  })
})
