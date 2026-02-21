import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect } from 'vitest'
import { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider } from './tooltip'

describe('Tooltip', () => {
  it('renders correctly when triggered', async () => {
    const user = userEvent.setup()
    render(
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <button>Hover me</button>
          </TooltipTrigger>
          <TooltipContent>
            Tooltip content
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    )
    
    const trigger = screen.getByText('Hover me')
    await user.hover(trigger)
    
    // Radix duplicates text for accessibility, find any one of them
    expect(await screen.findAllByText(/Tooltip content/i)).not.toHaveLength(0)
  })
})
