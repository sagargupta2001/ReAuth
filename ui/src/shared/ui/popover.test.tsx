import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { Popover, PopoverTrigger, PopoverContent } from './popover'

describe('Popover', () => {
  it('renders correctly when triggered', () => {
    render(
      <Popover>
        <PopoverTrigger asChild>
          <button>Open</button>
        </PopoverTrigger>
        <PopoverContent>
          <div>Popover content</div>
        </PopoverContent>
      </Popover>
    )
    
    expect(screen.queryByText('Popover content')).not.toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Open'))
    
    expect(screen.getByText('Popover content')).toBeInTheDocument()
  })
})
