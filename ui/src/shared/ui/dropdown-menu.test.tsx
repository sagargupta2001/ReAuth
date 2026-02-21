import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect } from 'vitest'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from './dropdown-menu'

describe('DropdownMenu', () => {
  it('renders correctly and opens', async () => {
    const user = userEvent.setup()
    render(
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button>Open Menu</button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem>Item 1</DropdownMenuItem>
          <DropdownMenuItem>Item 2</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    )
    
    expect(screen.queryByText('Item 1')).not.toBeInTheDocument()
    
    await user.click(screen.getByText('Open Menu'))
    
    expect(await screen.findByText('Item 1')).toBeInTheDocument()
    expect(await screen.findByText('Item 2')).toBeInTheDocument()
  })
})
