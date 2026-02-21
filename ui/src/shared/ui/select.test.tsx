import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from './select'

describe('Select', () => {
  it('renders correctly and opens', () => {
    render(
      <Select>
        <SelectTrigger aria-label="select-trigger">
          <SelectValue placeholder="Select an option" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="option1">Option 1</SelectItem>
          <SelectItem value="option2">Option 2</SelectItem>
        </SelectContent>
      </Select>
    )
    
    const trigger = screen.getByLabelText('select-trigger')
    expect(trigger).toBeInTheDocument()
    
    fireEvent.click(trigger)
    
    expect(screen.getByText('Option 1')).toBeInTheDocument()
    expect(screen.getByText('Option 2')).toBeInTheDocument()
  })
})
