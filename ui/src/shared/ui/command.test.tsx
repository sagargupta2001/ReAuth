import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Command, CommandInput, CommandList, CommandEmpty, CommandGroup, CommandItem } from './command'

describe('Command', () => {
  it('renders correctly', () => {
    render(
      <Command>
        <CommandInput placeholder="Type a command..." />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            <CommandItem>Item 1</CommandItem>
            <CommandItem>Item 2</CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    )
    
    expect(screen.getByPlaceholderText('Type a command...')).toBeInTheDocument()
    expect(screen.getByText('Suggestions')).toBeInTheDocument()
    expect(screen.getByText('Item 1')).toBeInTheDocument()
  })
})
