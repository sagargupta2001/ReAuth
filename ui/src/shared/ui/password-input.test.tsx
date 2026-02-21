import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { PasswordInput } from './password-input'

describe('PasswordInput', () => {
  it('toggles password visibility', () => {
    render(<PasswordInput placeholder="Password" />)
    const input = screen.getByPlaceholderText('Password') as HTMLInputElement
    
    expect(input.type).toBe('password')
    
    const toggle = screen.getByRole('button')
    fireEvent.click(toggle)
    
    expect(input.type).toBe('text')
    
    fireEvent.click(toggle)
    expect(input.type).toBe('password')
  })
})
